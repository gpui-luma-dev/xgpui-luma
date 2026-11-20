//! Text family: fields plus the shared editing engine.

mod editing;
mod runs;
mod state;
mod styled;
pub use styled::{StyledTextAreaTheme, StyledTextFieldTheme};

pub mod textarea;
pub mod textfield;

/// Default clipboard paste budget for TextField and TextArea, in UTF-8 bytes.
/// Override per control with `max_clipboard_paste_bytes`; `None` keeps the full clipboard.
pub const DEFAULT_MAX_CLIPBOARD_PASTE_BYTES: usize = 1_048_576;

pub(crate) use editing::{FocusNavigation, cap_clipboard_paste, handle_key_down, word_cluster_range};
pub(crate) use state::{EditableTextPolicy, TextSelectionState, select_all};
