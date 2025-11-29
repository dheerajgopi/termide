//! Editor module - core editor state and mode management
//!
//! This module provides the central `EditorState` that manages the overall editor state
//! including the current buffer, editing mode, status messages, and application lifecycle.
//!
//! # Examples
//!
//! ```no_run
//! use std::path::Path;
//! use termide::editor::{EditorState, EditorMode};
//! use termide::buffer::Position;
//!
//! # fn main() -> Result<(), anyhow::Error> {
//! // Create a new editor state
//! let mut state = EditorState::new();
//!
//! // Or load from a file
//! let mut state = EditorState::from_file(Path::new("example.txt"))?;
//!
//! // Switch modes
//! state.set_mode(EditorMode::Normal);
//!
//! // Insert characters (delegates to buffer)
//! state.handle_char_insert('H', Position::origin());
//!
//! // Save the buffer
//! state.save()?;
//! # Ok(())
//! # }
//! ```

mod editor_mode;
mod editor_state;

pub use editor_mode::EditorMode;
pub use editor_state::EditorState;

#[cfg(test)]
mod tests;
