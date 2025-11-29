//! Buffer struct for efficient text storage and manipulation using Rope

use ropey::Rope;
use std::path::PathBuf;

use super::Position;

/// The main text buffer using Rope for efficient text operations
///
/// This struct provides efficient text storage and manipulation using the ropey crate's
/// `Rope` data structure, which provides O(log n) complexity for insert and delete operations.
///
/// # Performance Characteristics
///
/// - Insert character: O(log n)
/// - Delete character: O(log n)
/// - Get line: O(log n)
/// - Line count: O(1)
///
/// where n is the total number of characters in the buffer.
///
/// # Examples
///
/// ```
/// use termide::buffer::{Buffer, Position};
///
/// // Create a new empty buffer
/// let mut buffer = Buffer::new();
///
/// // Insert characters
/// buffer.insert_char('H', Position::origin());
/// buffer.insert_char('i', Position::new(0, 1));
///
/// // Get content
/// assert_eq!(buffer.content(), "Hi");
/// assert!(buffer.is_dirty());
/// ```
#[derive(Debug, Clone)]
pub struct Buffer {
    /// The rope data structure storing the text content
    rope: Rope,
    /// Optional file path associated with this buffer
    file_path: Option<PathBuf>,
    /// Flag indicating if the buffer has unsaved changes
    dirty: bool,
}

impl Buffer {
    /// Creates a new empty buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::Buffer;
    ///
    /// let buffer = Buffer::new();
    /// assert_eq!(buffer.content(), "");
    /// assert!(!buffer.is_dirty());
    /// ```
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            file_path: None,
            dirty: false,
        }
    }

    /// Creates a buffer from a string
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::Buffer;
    ///
    /// let buffer = Buffer::from_str("Hello\nWorld");
    /// assert_eq!(buffer.line_count(), 2);
    /// assert_eq!(buffer.get_line(0), Some("Hello\n".to_string()));
    /// ```
    pub fn from_str(content: &str) -> Self {
        Self {
            rope: Rope::from_str(content),
            file_path: None,
            dirty: false,
        }
    }

    /// Inserts a character at the specified position
    ///
    /// Returns `true` if the insertion was successful, `false` if the position was invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position};
    ///
    /// let mut buffer = Buffer::new();
    /// assert!(buffer.insert_char('A', Position::origin()));
    /// assert_eq!(buffer.content(), "A");
    /// assert!(buffer.is_dirty());
    /// ```
    pub fn insert_char(&mut self, ch: char, pos: Position) -> bool {
        if let Some(char_idx) = self.position_to_char_idx(pos) {
            // Check if position is valid (within bounds or at end)
            if char_idx <= self.rope.len_chars() {
                self.rope.insert_char(char_idx, ch);
                self.dirty = true;
                return true;
            }
        }
        false
    }

    /// Deletes the character at the specified position
    ///
    /// Returns `true` if deletion was successful, `false` if position was invalid or no character exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position};
    ///
    /// let mut buffer = Buffer::from_str("ABC");
    /// assert!(buffer.delete_char_at(Position { line: 0, column: 1 }));
    /// assert_eq!(buffer.content(), "AC");
    /// ```
    pub fn delete_char_at(&mut self, pos: Position) -> bool {
        if let Some(char_idx) = self.position_to_char_idx(pos) {
            // Check if there's a character to delete
            if char_idx < self.rope.len_chars() {
                self.rope.remove(char_idx..char_idx + 1);
                self.dirty = true;
                return true;
            }
        }
        false
    }

    /// Returns the full buffer content as a String
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::Buffer;
    ///
    /// let buffer = Buffer::from_str("Hello\nWorld");
    /// assert_eq!(buffer.content(), "Hello\nWorld");
    /// ```
    pub fn content(&self) -> String {
        self.rope.to_string()
    }

    /// Returns the number of lines in the buffer
    ///
    /// Note: An empty buffer has 1 line, a buffer with one newline has 2 lines.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::Buffer;
    ///
    /// let buffer = Buffer::from_str("Line 1\nLine 2\nLine 3");
    /// assert_eq!(buffer.line_count(), 3);
    /// ```
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Returns the content of a specific line including the newline character
    ///
    /// Returns `None` if the line number is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::Buffer;
    ///
    /// let buffer = Buffer::from_str("First\nSecond\nThird");
    /// assert_eq!(buffer.get_line(0), Some("First\n".to_string()));
    /// assert_eq!(buffer.get_line(1), Some("Second\n".to_string()));
    /// assert_eq!(buffer.get_line(2), Some("Third".to_string()));
    /// assert_eq!(buffer.get_line(3), None);
    /// ```
    pub fn get_line(&self, line: usize) -> Option<String> {
        if line < self.rope.len_lines() {
            Some(self.rope.line(line).to_string())
        } else {
            None
        }
    }

    /// Returns the length of a specific line (excluding newline)
    ///
    /// Returns `None` if the line number is out of bounds.
    pub fn line_len(&self, line: usize) -> Option<usize> {
        if line < self.rope.len_lines() {
            let line_str = self.rope.line(line).to_string();
            // Remove newline for length calculation
            let len = line_str.trim_end_matches('\n').chars().count();
            Some(len)
        } else {
            None
        }
    }

    /// Checks if the buffer has unsaved changes
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position};
    ///
    /// let mut buffer = Buffer::new();
    /// assert!(!buffer.is_dirty());
    ///
    /// buffer.insert_char('X', Position::origin());
    /// assert!(buffer.is_dirty());
    ///
    /// buffer.clear_dirty();
    /// assert!(!buffer.is_dirty());
    /// ```
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clears the dirty flag (typically called after saving)
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Sets the dirty flag
    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    /// Gets the file path associated with this buffer
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    /// Sets the file path for this buffer
    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    /// Validates if a position is within buffer bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position};
    ///
    /// let buffer = Buffer::from_str("Hello");
    /// assert!(buffer.is_valid_position(Position { line: 0, column: 0 }));
    /// assert!(buffer.is_valid_position(Position { line: 0, column: 5 })); // At end
    /// assert!(!buffer.is_valid_position(Position { line: 0, column: 6 })); // Beyond end
    /// assert!(!buffer.is_valid_position(Position { line: 1, column: 0 })); // Invalid line
    /// ```
    pub fn is_valid_position(&self, pos: Position) -> bool {
        // Check if line is valid
        if pos.line >= self.rope.len_lines() {
            return false;
        }

        // Check if column is valid for this line
        if let Some(line_length) = self.line_len(pos.line) {
            // Allow position at end of line (for insertion)
            pos.column <= line_length
        } else {
            false
        }
    }

    /// Clamps a position to valid buffer bounds
    ///
    /// If the position is out of bounds, returns the nearest valid position.
    pub fn clamp_position(&self, pos: Position) -> Position {
        let max_line = self.rope.len_lines().saturating_sub(1);
        let line = pos.line.min(max_line);

        let max_column = self.line_len(line).unwrap_or(0);
        let column = pos.column.min(max_column);

        Position { line, column }
    }

    /// Converts a Position to a character index in the rope
    ///
    /// Returns None if the position is invalid
    fn position_to_char_idx(&self, pos: Position) -> Option<usize> {
        if pos.line >= self.rope.len_lines() {
            return None;
        }

        // Get the character index of the start of the line
        let line_start = self.rope.line_to_char(pos.line);

        // Get line content to validate column
        let line_str = self.rope.line(pos.line).to_string();
        let line_len = line_str.trim_end_matches('\n').chars().count();

        // Allow column at end of line (for insertion)
        if pos.column > line_len {
            return None;
        }

        Some(line_start + pos.column)
    }

    /// Returns the total number of characters in the buffer
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Checks if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
