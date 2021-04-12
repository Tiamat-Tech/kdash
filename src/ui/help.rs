use crate::app::DEFAULT_KEYBINDING;

pub fn get_help_docs() -> Vec<Vec<String>> {
  vec![
    vec![
      String::from("Jump to current play context"),
      DEFAULT_KEYBINDING.jump_to_all_context.to_string(),
      String::from("General"),
    ],
    vec![
      String::from("Move selection left"),
      String::from("h | <Left Arrow Key> | <Ctrl+b>"),
      String::from("General"),
    ],
    vec![
      String::from("Move selection down"),
      String::from("j | <Down Arrow Key> | <Ctrl+n>"),
      String::from("General"),
    ],
    vec![
      String::from("Move selection up"),
      String::from("k | <Up Arrow Key> | <Ctrl+p>"),
      String::from("General"),
    ],
    vec![
      String::from("Move selection right"),
      String::from("l | <Right Arrow Key> | <Ctrl+f>"),
      String::from("General"),
    ],
    vec![
      String::from("Move selection to top of list"),
      String::from("H"),
      String::from("General"),
    ],
    vec![
      String::from("Move selection to middle of list"),
      String::from("M"),
      String::from("General"),
    ],
    vec![
      String::from("Move selection to bottom of list"),
      String::from("L"),
      String::from("General"),
    ],
    vec![
      String::from("Enter active mode"),
      String::from("<Enter>"),
      String::from("General"),
    ],
    vec![
      String::from("Enter hover mode"),
      String::from("<Esc>"),
      String::from("Selected block"),
    ],
    vec![
      String::from("Save track in list or table"),
      String::from("s"),
      String::from("Selected block"),
    ],
    vec![
      String::from("Start playback or enter album/artist/playlist"),
      DEFAULT_KEYBINDING.submit.to_string(),
      String::from("Selected block"),
    ],
    vec![
      String::from("Play recommendations for song/artist"),
      String::from("r"),
      String::from("Selected block"),
    ],
    vec![
      String::from("Play all tracks for artist"),
      String::from("e"),
      String::from("Library -> Artists"),
    ],
    vec![
      String::from("Search with input text"),
      String::from("<Enter>"),
      String::from("Search input"),
    ],
    vec![
      String::from("Move cursor one space left"),
      String::from("<Left Arrow Key>"),
      String::from("Search input"),
    ],
    vec![
      String::from("Move cursor one space right"),
      String::from("<Right Arrow Key>"),
      String::from("Search input"),
    ],
    vec![
      String::from("Delete entire input"),
      String::from("<Ctrl+l>"),
      String::from("Search input"),
    ],
    vec![
      String::from("Delete text from cursor to start of input"),
      String::from("<Ctrl+u>"),
      String::from("Search input"),
    ],
    vec![
      String::from("Delete text from cursor to end of input"),
      String::from("<Ctrl+k>"),
      String::from("Search input"),
    ],
    vec![
      String::from("Delete previous word"),
      String::from("<Ctrl+w>"),
      String::from("Search input"),
    ],
    vec![
      String::from("Jump to start of input"),
      String::from("<Ctrl+a>"),
      String::from("Search input"),
    ],
    vec![
      String::from("Jump to end of input"),
      String::from("<Ctrl+e>"),
      String::from("Search input"),
    ],
    vec![
      String::from("Escape from the input back to hovered block"),
      String::from("<Esc>"),
      String::from("Search input"),
    ],
    vec![
      String::from("Delete saved album"),
      String::from("D"),
      String::from("Library -> Albums"),
    ],
    vec![
      String::from("Delete saved playlist"),
      String::from("D"),
      String::from("Playlist"),
    ],
    vec![
      String::from("Follow an artist/playlist"),
      String::from("w"),
      String::from("Search result"),
    ],
    vec![
      String::from("Save (like) album to library"),
      String::from("w"),
      String::from("Search result"),
    ],
    vec![
      String::from("Play random song in playlist"),
      String::from("S"),
      String::from("Selected Playlist"),
    ],
    vec![
      String::from("Toggle sort order of podcast episodes"),
      String::from("S"),
      String::from("Selected Show"),
    ],
  ]
}