#![warn(rust_2018_idioms)]
#[deny(clippy::shadow_unrelated)]
mod app;
mod banner;
mod cmd;
mod event;
mod handlers;
mod network;
mod ui;

use std::{
  fs::File,
  io::{self, stdout, Stdout},
  panic::{self, PanicInfo},
  sync::Arc,
};

use anyhow::{anyhow, Result};
use app::App;
use banner::BANNER;
use clap::{builder::PossibleValuesParser, Parser};
use cmd::{CmdRunner, IoCmdEvent};
use crossterm::{
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::Key;
use k8s_openapi::chrono::{self};
use log::{info, warn, LevelFilter, SetLoggerError};
use network::{
  get_client,
  stream::{IoStreamEvent, NetworkStream},
  IoEvent, Network,
};
use ratatui::{
  backend::{Backend, CrosstermBackend},
  Terminal,
};
use simplelog::{Config, WriteLogger};
use tokio::sync::{mpsc, Mutex};

/// kdash CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, override_usage = "Press `?` while running the app to see keybindings", before_help = BANNER)]
pub struct Cli {
  /// Set the tick rate (milliseconds): the lower the number the higher the FPS.
  #[arg(short, long, value_parser, default_value_t = 250)]
  pub tick_rate: u64,
  /// Set the network call polling rate (milliseconds, should be multiples of tick-rate):
  /// the lower the number the higher the network calls.
  #[arg(short, long, value_parser, default_value_t = 5000)]
  pub poll_rate: u64,
  /// whether unicode symbols are used to improve the overall look of the app
  #[arg(short, long, value_parser, default_value_t = true)]
  pub enhanced_graphics: bool,
  /// Enables debug mode and writes logs to 'kdash-debug-<timestamp>.log' file in the current directory.
  /// Default behavior is to write INFO logs. Pass a log level to overwrite the default.
  #[arg(
    name = "debug",
    short,
    long,
    default_missing_value = "Info",
    require_equals = true,
    num_args = 0..=1,
    ignore_case = true,
    value_parser = PossibleValuesParser::new(&["info", "debug", "trace", "warn", "error"])
  )]
  pub debug: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
  openssl_probe::init_ssl_cert_env_vars();
  panic::set_hook(Box::new(|info| {
    panic_hook(info);
  }));

  // parse CLI arguments
  let cli = Cli::parse();

  // Setup logging if debug flag is set
  if cli.debug.is_some() {
    setup_logging(cli.debug.clone())?;
    info!(
      "Debug mode is enabled. Level: {}, KDash version: {}",
      cli.debug.clone().unwrap(),
      env!("CARGO_PKG_VERSION")
    );
  }

  if cli.tick_rate >= 1000 {
    panic!("Tick rate must be below 1000");
  }
  if (cli.poll_rate % cli.tick_rate) > 0u64 {
    panic!("Poll rate must be multiple of tick-rate");
  }

  // channels for communication between network/cmd threads & UI thread
  let (sync_io_tx, sync_io_rx) = mpsc::channel::<IoEvent>(500);
  let (sync_io_stream_tx, sync_io_stream_rx) = mpsc::channel::<IoStreamEvent>(500);
  let (sync_io_cmd_tx, sync_io_cmd_rx) = mpsc::channel::<IoCmdEvent>(500);

  // Initialize app state
  let app = Arc::new(Mutex::new(App::new(
    sync_io_tx,
    sync_io_stream_tx,
    sync_io_cmd_tx,
    cli.enhanced_graphics,
    cli.poll_rate / cli.tick_rate,
  )));

  // make copies for the network/cli threads
  let app_nw = Arc::clone(&app);
  let app_stream = Arc::clone(&app);
  let app_cli = Arc::clone(&app);

  // Launch network thread
  std::thread::spawn(move || {
    info!("Starting network thread");
    start_network(sync_io_rx, &app_nw);
  });
  // Launch network thread for streams
  std::thread::spawn(move || {
    info!("Starting network stream thread");
    start_stream_network(sync_io_stream_rx, &app_stream);
  });
  // Launch thread for cmd runner
  std::thread::spawn(move || {
    info!("Starting cmd runner thread");
    start_cmd_runner(sync_io_cmd_rx, &app_cli);
  });
  // Launch the UI asynchronously
  // The UI must run in the "main" thread
  start_ui(cli, &app).await?;

  Ok(())
}

#[tokio::main]
async fn start_network(mut io_rx: mpsc::Receiver<IoEvent>, app: &Arc<Mutex<App>>) {
  match get_client(None).await {
    Ok(client) => {
      let mut network = Network::new(client, app);

      while let Some(io_event) = io_rx.recv().await {
        info!("Network event received: {:?}", io_event);
        network.handle_network_event(io_event).await;
      }
    }
    Err(e) => {
      let mut app = app.lock().await;
      app.handle_error(anyhow!("Unable to obtain Kubernetes client. {:?}", e));
    }
  }
}

#[tokio::main]
async fn start_stream_network(mut io_rx: mpsc::Receiver<IoStreamEvent>, app: &Arc<Mutex<App>>) {
  match get_client(None).await {
    Ok(client) => {
      let mut network = NetworkStream::new(client, app);

      while let Some(io_event) = io_rx.recv().await {
        info!("Network stream event received: {:?}", io_event);
        network.handle_network_stream_event(io_event).await;
      }
    }
    Err(e) => {
      let mut app = app.lock().await;
      app.handle_error(anyhow!("Unable to obtain Kubernetes client. {:?}", e));
    }
  }
}

#[tokio::main]
async fn start_cmd_runner(mut io_rx: mpsc::Receiver<IoCmdEvent>, app: &Arc<Mutex<App>>) {
  let mut cmd = CmdRunner::new(app);

  while let Some(io_event) = io_rx.recv().await {
    info!("Cmd event received: {:?}", io_event);
    cmd.handle_cmd_event(io_event).await;
  }
}

async fn start_ui(cli: Cli, app: &Arc<Mutex<App>>) -> Result<()> {
  info!("Starting UI");
  // see https://docs.rs/crossterm/0.17.7/crossterm/terminal/#raw-mode
  enable_raw_mode()?;
  // Terminal initialization
  let mut stdout = stdout();
  // not capturing mouse to make text select/copy possible
  execute!(stdout, EnterAlternateScreen)?;
  // terminal backend for cross platform support
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.clear()?;
  terminal.hide_cursor()?;
  // custom events
  let events = event::Events::new(cli.tick_rate);
  let mut is_first_render = true;
  // main UI loop
  loop {
    let mut app = app.lock().await;
    // Get the size of the screen on each loop to account for resize event
    if let Ok(size) = terminal.backend().size() {
      // Reset the help menu if the terminal was resized
      if app.refresh || app.size != size {
        app.size = size;
      }
    };

    // draw the UI layout
    terminal.draw(|f| ui::draw(f, &mut app))?;

    // handle key events
    match events.next()? {
      event::Event::Input(key_event) => {
        info!("Input event received: {:?}", key_event);
        // quit on CTRL + C
        let key = Key::from(key_event);

        if key == Key::Ctrl('c') {
          break;
        }
        // handle all other keys
        handlers::handle_key_events(key, key_event, &mut app).await
      }
      // handle mouse events
      event::Event::MouseInput(mouse) => handlers::handle_mouse_events(mouse, &mut app).await,
      // handle tick events
      event::Event::Tick => {
        app.on_tick(is_first_render).await;
      }
    }

    is_first_render = false;

    if app.should_quit {
      break;
    }
  }

  terminal.show_cursor()?;
  shutdown(terminal)?;
  Ok(())
}

// shutdown the CLI and show terminal
fn shutdown(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
  info!("Shutting down");
  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
  terminal.show_cursor()?;
  Ok(())
}

fn setup_logging(debug: Option<String>) -> Result<(), SetLoggerError> {
  let log_file = format!(
    "./kdash-debug-{}.log",
    chrono::Local::now().format("%Y%m%d%H%M%S")
  );
  let log_level = debug
    .map(|level| match level.to_lowercase().as_str() {
      "debug" => LevelFilter::Debug,
      "trace" => LevelFilter::Trace,
      "warn" => LevelFilter::Warn,
      "error" => LevelFilter::Error,
      _ => LevelFilter::Info,
    })
    .unwrap_or_else(|| LevelFilter::Info);

  WriteLogger::init(
    log_level,
    Config::default(),
    File::create(log_file).unwrap(),
  )
}

#[cfg(debug_assertions)]
fn panic_hook(info: &PanicInfo<'_>) {
  use backtrace::Backtrace;
  use crossterm::style::Print;

  let (msg, location) = get_panic_info(info);

  let stacktrace: String = format!("{:?}", Backtrace::new()).replace('\n', "\n\r");

  disable_raw_mode().unwrap();
  execute!(
    io::stdout(),
    LeaveAlternateScreen,
    Print(format!(
      "thread '<unnamed>' panicked at '{}', {}\n\r{}",
      msg, location, stacktrace
    )),
  )
  .unwrap();
}

#[cfg(not(debug_assertions))]
fn panic_hook(info: &PanicInfo<'_>) {
  use crossterm::style::Print;
  use human_panic::{handle_dump, print_msg, Metadata};
  use log::error;

  let meta = Metadata {
    version: env!("CARGO_PKG_VERSION").into(),
    name: env!("CARGO_PKG_NAME").into(),
    authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
    homepage: env!("CARGO_PKG_HOMEPAGE").into(),
  };
  let file_path = handle_dump(&meta, info);
  let (msg, location) = get_panic_info(info);

  error!(
    "thread '<unnamed>' panicked at '{}', {}\n\r{}",
    msg, location, stacktrace
  );

  disable_raw_mode().unwrap();
  execute!(
    io::stdout(),
    LeaveAlternateScreen,
    Print(format!("Error: '{}' at {}\n", msg, location)),
  )
  .unwrap();
  print_msg(file_path, &meta).expect("human-panic: printing error message to console failed");
}

fn get_panic_info(info: &PanicInfo<'_>) -> (String, String) {
  let location = info.location().unwrap();

  let msg = match info.payload().downcast_ref::<&'static str>() {
    Some(s) => *s,
    None => match info.payload().downcast_ref::<String>() {
      Some(s) => &s[..],
      None => "Box<Any>",
    },
  };

  (msg.to_string(), format!("{}", location))
}
