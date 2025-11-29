//! Editor state management

use std::path::Path;

use anyhow::{Context, Result};

use crate::buffer::{Buffer, Position};
use crate::file_io::{read_file, write_file};

use super::EditorMode;

/// Central editor state managing buffer, mode, and UI state
///
/// `EditorState` is the core component that ties together the buffer,
/// editing mode, status messages, and application lifecycle management.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use termide::editor::{EditorState, EditorMode};
/// use termide::buffer::Position;
///
/// # fn main() -> Result<(), anyhow::Error> {
/// // Create new empty editor
/// let mut state = EditorState::new();
///
/// // Insert some text
/// state.handle_char_insert('H', Position::origin());
/// state.handle_char_insert('i', Position::new(0, 1));
///
/// // Save to file
/// state.set_file_path(Path::new("test.txt"));
/// state.save()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct EditorState {
    /// The text buffer
    buffer: Buffer,
    /// Current editing mode
    mode: EditorMode,
    /// Status message to display to user
    status_message: Option<String>,
    /// Flag indicating the editor should quit
    should_quit: bool,
}

impl EditorState {
    /// Creates a new editor state with an empty buffer in Insert mode
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::editor::{EditorState, EditorMode};
    ///
    /// let state = EditorState::new();
    /// assert_eq!(state.buffer().content(), "");
    /// assert!(!state.buffer().is_dirty());
    /// assert_eq!(state.mode(), EditorMode::Insert);
    /// ```
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            mode: EditorMode::Insert,
            status_message: None,
            should_quit: false,
        }
    }

    /// Creates a new editor state by loading a file
    ///
    /// If the file exists, its contents are loaded into the buffer.
    /// If the file doesn't exist, creates a new buffer with that file path
    /// (the file will be created on save).
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read (permissions, invalid UTF-8, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use termide::editor::EditorState;
    ///
    /// # fn main() -> Result<(), anyhow::Error> {
    /// // Load existing file
    /// let state = EditorState::from_file(Path::new("existing.txt"))?;
    ///
    /// // New file (doesn't exist yet)
    /// let state = EditorState::from_file(Path::new("new.txt"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file(path: &Path) -> Result<Self> {
        let mut buffer = if path.exists() {
            let content = read_file(path)
                .with_context(|| format!("Failed to load file: {}", path.display()))?;
            Buffer::from_str(&content)
        } else {
            Buffer::new()
        };

        buffer.set_file_path(path.to_path_buf());

        Ok(Self {
            buffer,
            mode: EditorMode::Insert,
            status_message: None,
            should_quit: false,
        })
    }

    /// Returns a reference to the buffer
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Returns a mutable reference to the buffer
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Returns the current editing mode
    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    /// Sets the editing mode
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::editor::{EditorState, EditorMode};
    ///
    /// let mut state = EditorState::new();
    /// assert_eq!(state.mode(), EditorMode::Insert);
    ///
    /// state.set_mode(EditorMode::Normal);
    /// assert_eq!(state.mode(), EditorMode::Normal);
    /// ```
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    /// Returns the current status message, if any
    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    /// Sets a status message to display to the user
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::editor::EditorState;
    ///
    /// let mut state = EditorState::new();
    /// state.set_status_message("File saved successfully".to_string());
    /// assert_eq!(state.status_message(), Some("File saved successfully"));
    /// ```
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clears the status message
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Returns whether the editor should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Sets the quit flag
    ///
    /// This method checks if the buffer has unsaved changes and returns
    /// whether the quit should proceed.
    ///
    /// Returns `true` if quit should proceed, `false` if there are unsaved changes
    /// that need to be handled.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::editor::EditorState;
    /// use termide::buffer::Position;
    ///
    /// let mut state = EditorState::new();
    ///
    /// // No changes, can quit
    /// assert!(state.request_quit());
    /// assert!(state.should_quit());
    ///
    /// // Reset state
    /// let mut state = EditorState::new();
    /// state.handle_char_insert('A', Position::origin());
    ///
    /// // Has changes, first attempt warns
    /// assert!(!state.request_quit());
    /// assert_eq!(state.status_message(), Some("Warning: Unsaved changes! Press Ctrl+Q again to force quit."));
    ///
    /// // Second attempt forces quit
    /// assert!(state.request_quit());
    /// assert!(state.should_quit());
    /// ```
    pub fn request_quit(&mut self) -> bool {
        if self.buffer.is_dirty() {
            // If already warned (status message exists), force quit
            if self.status_message.is_some() {
                self.should_quit = true;
                true
            } else {
                // First attempt - warn user
                self.set_status_message(
                    "Warning: Unsaved changes! Press Ctrl+Q again to force quit.".to_string(),
                );
                false
            }
        } else {
            // No unsaved changes, quit immediately
            self.should_quit = true;
            true
        }
    }

    /// Forces quit without checking for unsaved changes
    pub fn force_quit(&mut self) {
        self.should_quit = true;
    }

    /// Inserts a character at the specified position
    ///
    /// This delegates to the buffer's insert_char method.
    ///
    /// Returns `true` if insertion was successful.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::editor::EditorState;
    /// use termide::buffer::Position;
    ///
    /// let mut state = EditorState::new();
    /// assert!(state.handle_char_insert('A', Position::origin()));
    /// assert_eq!(state.buffer().content(), "A");
    /// ```
    pub fn handle_char_insert(&mut self, ch: char, pos: Position) -> bool {
        self.buffer.insert_char(ch, pos)
    }

    /// Deletes a character at the specified position
    ///
    /// Returns `true` if deletion was successful.
    pub fn handle_char_delete(&mut self, pos: Position) -> bool {
        self.buffer.delete_char_at(pos)
    }

    /// Sets the file path for the buffer
    pub fn set_file_path(&mut self, path: &Path) {
        self.buffer.set_file_path(path.to_path_buf());
    }

    /// Saves the buffer to its associated file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No file path is associated with the buffer
    /// - The file cannot be written (permissions, disk full, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use termide::editor::EditorState;
    /// use termide::buffer::Position;
    ///
    /// # fn main() -> Result<(), anyhow::Error> {
    /// let mut state = EditorState::new();
    /// state.set_file_path(Path::new("test.txt"));
    /// state.handle_char_insert('A', Position::origin());
    ///
    /// state.save()?;
    /// assert!(!state.buffer().is_dirty());
    /// assert_eq!(state.status_message(), Some("Saved successfully"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn save(&mut self) -> Result<()> {
        let path = self
            .buffer
            .file_path()
            .ok_or_else(|| anyhow::anyhow!("No file path associated with buffer"))?;

        write_file(path, &self.buffer.content())
            .with_context(|| format!("Failed to save file: {}", path.display()))?;

        self.buffer.clear_dirty();
        self.set_status_message("Saved successfully".to_string());

        Ok(())
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}
