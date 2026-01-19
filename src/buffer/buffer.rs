//! Buffer struct for efficient text storage and manipulation using Rope

use ropey::Rope;
use std::path::PathBuf;

use super::Position;
use super::Selection;

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
    /// Current text selection (transient, not persisted to disk)
    selection: Option<Selection>,
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
            selection: None,
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
            selection: None,
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

    /// Deletes the character at the cursor position (forward delete)
    ///
    /// This is different from backspace - it deletes the character at the cursor,
    /// not the character before it. If at the end of a line, it joins with the next line.
    ///
    /// Returns `true` if deletion was successful, `false` if at end of buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position};
    ///
    /// let mut buffer = Buffer::from_str("ABC");
    /// assert!(buffer.delete_forward(Position { line: 0, column: 0 }));
    /// assert_eq!(buffer.content(), "BC");
    ///
    /// // At end of line, joins with next line
    /// let mut buffer = Buffer::from_str("Line1\nLine2");
    /// assert!(buffer.delete_forward(Position { line: 0, column: 5 }));
    /// assert_eq!(buffer.content(), "Line1Line2");
    /// ```
    pub fn delete_forward(&mut self, pos: Position) -> bool {
        self.delete_char_at(pos)
    }

    /// Calculates the column position for the start of a line (column 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position};
    ///
    /// let buffer = Buffer::from_str("Hello World");
    /// let start_pos = buffer.get_line_start(Position { line: 0, column: 5 });
    /// assert_eq!(start_pos, Position { line: 0, column: 0 });
    /// ```
    pub fn get_line_start(&self, pos: Position) -> Position {
        Position {
            line: pos.line,
            column: 0,
        }
    }

    /// Calculates the column position for the end of the current line
    ///
    /// Returns a position at the last character of the line (excluding newline).
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position};
    ///
    /// let buffer = Buffer::from_str("Hello");
    /// let end_pos = buffer.get_line_end(Position { line: 0, column: 0 });
    /// assert_eq!(end_pos, Position { line: 0, column: 5 });
    ///
    /// let buffer = Buffer::from_str("Hello\nWorld");
    /// let end_pos = buffer.get_line_end(Position { line: 0, column: 2 });
    /// assert_eq!(end_pos, Position { line: 0, column: 5 });
    /// ```
    pub fn get_line_end(&self, pos: Position) -> Position {
        let line_len = self.line_len(pos.line).unwrap_or(0);
        Position {
            line: pos.line,
            column: line_len,
        }
    }

    /// Calculates the position after moving up by viewport_height lines
    ///
    /// Clamps to the first line if moving beyond buffer start.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position};
    ///
    /// let buffer = Buffer::from_str("Line1\nLine2\nLine3\nLine4\nLine5");
    /// let new_pos = buffer.page_up(Position { line: 4, column: 0 }, 2);
    /// assert_eq!(new_pos, Position { line: 2, column: 0 });
    ///
    /// // Clamps to first line
    /// let new_pos = buffer.page_up(Position { line: 1, column: 0 }, 5);
    /// assert_eq!(new_pos, Position { line: 0, column: 0 });
    /// ```
    pub fn page_up(&self, pos: Position, viewport_height: usize) -> Position {
        let new_line = pos.line.saturating_sub(viewport_height);
        let column = self.clamp_column_to_line(new_line, pos.column);
        Position {
            line: new_line,
            column,
        }
    }

    /// Calculates the position after moving down by viewport_height lines
    ///
    /// Clamps to the last line if moving beyond buffer end.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position};
    ///
    /// let buffer = Buffer::from_str("Line1\nLine2\nLine3\nLine4\nLine5");
    /// let new_pos = buffer.page_down(Position { line: 0, column: 0 }, 2);
    /// assert_eq!(new_pos, Position { line: 2, column: 0 });
    ///
    /// // Clamps to last line
    /// let new_pos = buffer.page_down(Position { line: 3, column: 0 }, 5);
    /// assert_eq!(new_pos, Position { line: 4, column: 0 });
    /// ```
    pub fn page_down(&self, pos: Position, viewport_height: usize) -> Position {
        let max_line = self.rope.len_lines().saturating_sub(1);
        let new_line = (pos.line + viewport_height).min(max_line);
        let column = self.clamp_column_to_line(new_line, pos.column);
        Position {
            line: new_line,
            column,
        }
    }

    /// Clamps a column to the length of a specific line
    ///
    /// Used internally for vertical navigation to keep cursor in valid positions.
    fn clamp_column_to_line(&self, line: usize, column: usize) -> usize {
        self.line_len(line)
            .map(|len| column.min(len))
            .unwrap_or(0)
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

    // ==================== Selection Methods ====================

    /// Returns a reference to the current selection, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position, Selection};
    ///
    /// let mut buffer = Buffer::from_str("Hello World");
    /// assert!(buffer.selection().is_none());
    ///
    /// buffer.set_selection(Some(Selection::with_anchor_and_cursor(
    ///     Position::new(0, 0),
    ///     Position::new(0, 5)
    /// )));
    /// assert!(buffer.selection().is_some());
    /// ```
    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    /// Sets the current selection.
    ///
    /// Pass `None` to clear the selection. Selection state is transient
    /// and is not persisted when the buffer is saved to disk.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position, Selection};
    ///
    /// let mut buffer = Buffer::from_str("Hello World");
    ///
    /// // Set a selection
    /// buffer.set_selection(Some(Selection::with_anchor_and_cursor(
    ///     Position::new(0, 0),
    ///     Position::new(0, 5)
    /// )));
    /// assert!(buffer.has_selection());
    ///
    /// // Clear the selection
    /// buffer.set_selection(None);
    /// assert!(!buffer.has_selection());
    /// ```
    pub fn set_selection(&mut self, selection: Option<Selection>) {
        self.selection = selection;
    }

    /// Returns true if there is an active selection with text selected.
    ///
    /// Returns false if there is no selection or if the selection is collapsed
    /// (anchor equals cursor).
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position, Selection};
    ///
    /// let mut buffer = Buffer::from_str("Hello");
    /// assert!(!buffer.has_selection());
    ///
    /// // Collapsed selection (no text selected)
    /// buffer.set_selection(Some(Selection::new(Position::origin())));
    /// assert!(!buffer.has_selection());
    ///
    /// // Active selection with text
    /// buffer.set_selection(Some(Selection::with_anchor_and_cursor(
    ///     Position::new(0, 0),
    ///     Position::new(0, 5)
    /// )));
    /// assert!(buffer.has_selection());
    /// ```
    pub fn has_selection(&self) -> bool {
        self.selection
            .as_ref()
            .map(|s| s.has_selection())
            .unwrap_or(false)
    }

    /// Returns the selected text as a String, if there is an active selection.
    ///
    /// Returns `None` if there is no selection or if the selection is collapsed.
    /// Multi-line selections include newline characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position, Selection};
    ///
    /// let mut buffer = Buffer::from_str("Hello World");
    /// buffer.set_selection(Some(Selection::with_anchor_and_cursor(
    ///     Position::new(0, 0),
    ///     Position::new(0, 5)
    /// )));
    /// assert_eq!(buffer.selected_text(), Some("Hello".to_string()));
    ///
    /// // Multi-line selection
    /// let mut buffer = Buffer::from_str("Line1\nLine2\nLine3");
    /// buffer.set_selection(Some(Selection::with_anchor_and_cursor(
    ///     Position::new(0, 0),
    ///     Position::new(1, 5)
    /// )));
    /// assert_eq!(buffer.selected_text(), Some("Line1\nLine2".to_string()));
    /// ```
    pub fn selected_text(&self) -> Option<String> {
        let selection = self.selection.as_ref()?;
        if !selection.has_selection() {
            return None;
        }

        let (start, end) = selection.range();

        // Clamp positions to buffer bounds
        let start = self.clamp_position(start);
        let end = self.clamp_position(end);

        // Convert positions to character indices
        let start_idx = self.position_to_char_idx(start)?;
        let end_idx = self.position_to_char_idx(end)?;

        // Handle edge case where end is at EOF
        let end_idx = end_idx.min(self.rope.len_chars());

        if start_idx >= end_idx {
            return None;
        }

        Some(self.rope.slice(start_idx..end_idx).to_string())
    }

    /// Deletes the selected text range and clears the selection.
    ///
    /// Returns `true` if text was deleted, `false` if there was no selection
    /// or the operation failed. Sets the dirty flag if text was deleted.
    ///
    /// Note: Currently does not integrate with undo/redo system as it hasn't
    /// been implemented yet. Undo integration will be added in a future task.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position, Selection};
    ///
    /// let mut buffer = Buffer::from_str("Hello World");
    /// buffer.set_selection(Some(Selection::with_anchor_and_cursor(
    ///     Position::new(0, 0),
    ///     Position::new(0, 5)
    /// )));
    ///
    /// assert!(buffer.delete_selection());
    /// assert_eq!(buffer.content(), " World");
    /// assert!(buffer.is_dirty());
    /// assert!(!buffer.has_selection());
    ///
    /// // No selection - returns false
    /// assert!(!buffer.delete_selection());
    /// ```
    pub fn delete_selection(&mut self) -> bool {
        let selection = match self.selection.take() {
            Some(s) if s.has_selection() => s,
            _ => return false,
        };

        let (start, end) = selection.range();

        // Clamp positions to buffer bounds
        let start = self.clamp_position(start);
        let end = self.clamp_position(end);

        // Convert positions to character indices
        let start_idx = match self.position_to_char_idx(start) {
            Some(idx) => idx,
            None => return false,
        };

        let end_idx = match self.position_to_char_idx(end) {
            Some(idx) => idx,
            None => return false,
        };

        // Handle edge case where end is at EOF
        let end_idx = end_idx.min(self.rope.len_chars());

        if start_idx >= end_idx {
            return false;
        }

        // Delete the selected range
        self.rope.remove(start_idx..end_idx);
        self.dirty = true;

        // Selection is already cleared (we used take())
        true
    }

    /// Returns the position of the buffer end (last line, last column).
    ///
    /// Useful for operations like "select all" that need to know the buffer bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Buffer, Position};
    ///
    /// let buffer = Buffer::from_str("Hello\nWorld");
    /// assert_eq!(buffer.end_position(), Position::new(1, 5));
    ///
    /// let empty = Buffer::new();
    /// assert_eq!(empty.end_position(), Position::new(0, 0));
    /// ```
    pub fn end_position(&self) -> Position {
        let last_line = self.rope.len_lines().saturating_sub(1);
        let last_column = self.line_len(last_line).unwrap_or(0);
        Position::new(last_line, last_column)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
