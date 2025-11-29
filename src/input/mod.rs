//! Input module - keyboard input handling and command mapping
//!
//! This module processes keyboard events using Crossterm and maps them to
//! editor commands based on the current mode.
//!
//! # Key Mappings
//!
//! ## Insert Mode
//! - Printable characters → Insert at cursor
//! - `Backspace` → Delete character before cursor
//! - `Enter` → Insert newline
//! - `Esc` → Switch to Normal mode
//! - `Ctrl+S` → Save file
//! - `Ctrl+Q` → Quit editor
//!
//! ## Normal Mode
//! - `i` → Switch to Insert mode
//! - Arrow keys → Move cursor
//! - `Ctrl+S` → Save file
//! - `Ctrl+Q` → Quit editor
//!
//! # Examples
//!
//! ```
//! use termide::input::{handle_key_event, EditorCommand, Direction};
//! use termide::editor::EditorMode;
//! use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
//!
//! // Insert mode: typing 'a' inserts character
//! let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
//! let command = handle_key_event(event, EditorMode::Insert);
//! assert_eq!(command, Some(EditorCommand::InsertChar('a')));
//!
//! // Normal mode: 'i' switches to insert mode
//! let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
//! let command = handle_key_event(event, EditorMode::Normal);
//! assert_eq!(command, Some(EditorCommand::ChangeMode(EditorMode::Insert)));
//!
//! // Ctrl+S saves in both modes
//! let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
//! let command = handle_key_event(event, EditorMode::Insert);
//! assert_eq!(command, Some(EditorCommand::Save));
//! ```

mod command;
mod direction;
mod handler;

pub use command::EditorCommand;
pub use direction::Direction;
pub use handler::handle_key_event;

#[cfg(test)]
mod tests;
