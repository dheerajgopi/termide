//! UI module - terminal UI rendering with Ratatui
//!
//! This module handles rendering the editor interface including text area,
//! status bar, and status messages.
//!
//! # Architecture
//!
//! The renderer uses Ratatui for terminal UI rendering with the following features:
//! - Viewport optimization: only visible lines are rendered
//! - Frame skipping: unchanged frames are not redrawn (dirty checking)
//! - Automatic scrolling: keeps cursor in view
//! - Status bar with file info, mode, and position
//! - Status messages with color coding
//!
//! # Examples
//!
//! ```no_run
//! use termide::ui::Renderer;
//! use termide::editor::EditorState;
//! use termide::buffer::Position;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut renderer = Renderer::new()?;
//! let state = EditorState::new();
//! let cursor = Position::origin();
//!
//! // Render the current state
//! renderer.render(&state, cursor)?;
//!
//! // Clean up when done
//! renderer.restore_terminal()?;
//! # Ok(())
//! # }
//! ```

mod renderer;

pub use renderer::Renderer;

#[cfg(test)]
mod tests;