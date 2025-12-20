//! Buffer module - text buffer management using Rope data structure
//!
//! This module provides efficient text storage and manipulation using the ropey crate.
//! The `Buffer` struct uses a `Rope` for O(log n) insert and delete operations, making it
//! suitable for editing large files efficiently.
//!
//! # Examples
//!
//! ```
//! use termide::buffer::{Buffer, Position};
//!
//! // Create a new empty buffer
//! let mut buffer = Buffer::new();
//!
//! // Insert characters
//! buffer.insert_char('H', Position { line: 0, column: 0 });
//! buffer.insert_char('i', Position { line: 0, column: 1 });
//!
//! // Get content
//! assert_eq!(buffer.content(), "Hi");
//! ```

mod buffer;
mod position;
mod selection;

pub use buffer::Buffer;
pub use position::Position;
pub use selection::Selection;

#[cfg(test)]
mod tests;
