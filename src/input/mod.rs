//! Input module - keyboard input handling and command mapping
//!
//! This module processes keyboard events using Crossterm and maps them to
//! editor commands based on the current mode using a flexible keybinding registry.
//!
//! # Architecture
//!
//! The input system uses:
//! - `InputHandler`: Main entry point for processing keyboard events
//! - `KeyBindingRegistry`: Stores and matches keybindings with priority ordering
//! - `EditorCommand`: Commands that can be executed by the editor
//! - Default bindings: Registered automatically at startup
//!
//! # Key Mappings
//!
//! ## Insert Mode
//! - Printable characters → Insert at cursor
//! - `Backspace` → Delete character before cursor
//! - `Enter` → Insert newline
//! - `Esc` → Switch to Normal mode
//! - Arrow keys → Move cursor
//! - `Ctrl+S` → Save file
//! - `Ctrl+Q` → Quit editor
//!
//! ## Normal Mode
//! - `i` → Switch to Insert mode
//! - Arrow keys → Move cursor
//! - `Ctrl+S` → Save file
//! - `Ctrl+Q` → Quit editor
//!
//! ## Prompt Mode
//! - Printable characters → Insert into prompt
//! - `Backspace` → Delete from prompt
//! - `Enter` → Accept prompt
//! - `Esc` → Cancel prompt
//!
//! # Examples
//!
//! ```
//! use termide::input::input_handler::{InputHandler, MatchResult};
//! use termide::input::{EditorCommand, Direction};
//! use termide::editor::EditorMode;
//! use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
//!
//! // Create and use the input handler
//! let mut handler = InputHandler::new();
//! let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
//! let result = handler.process_key_event(event, EditorMode::Normal);
//!
//! match result {
//!     MatchResult::Matched(cmd) => {
//!         // Execute the command
//!         assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Insert));
//!     }
//!     MatchResult::Partial => {
//!         // Wait for next key in sequence
//!     }
//!     MatchResult::NoMatch => {
//!         // No binding found, use default behavior
//!     }
//! }
//! ```

pub mod bindings;
mod command;
pub mod config;
mod direction;
pub mod input_handler;
pub mod keybinding;
pub mod registry;

pub use command::{EditorCommand, CommandParseError};
pub use direction::Direction;
pub use keybinding::ParseError;

#[cfg(test)]
mod tests;
