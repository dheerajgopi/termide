//! Editor mode enumeration

/// Represents the current editing mode of the editor
///
/// The editor supports three modes:
/// - `Insert`: Characters typed are inserted into the buffer
/// - `Normal`: Navigation and commands (vi-like behavior)
/// - `Prompt`: Prompting the user for input (e.g., filename)
///
/// # Examples
///
/// ```
/// use termide::editor::EditorMode;
///
/// let mode = EditorMode::Insert;
/// assert_eq!(mode.to_string(), "INSERT");
///
/// let mode = EditorMode::Normal;
/// assert_eq!(mode.to_string(), "NORMAL");
///
/// let mode = EditorMode::Prompt;
/// assert_eq!(mode.to_string(), "PROMPT");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Insert mode - characters are inserted at the cursor
    Insert,
    /// Normal mode - navigation and commands
    Normal,
    /// Prompt mode - user is being prompted for input
    Prompt,
}

impl EditorMode {
    /// Returns the string representation of the mode for display
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::editor::EditorMode;
    ///
    /// assert_eq!(EditorMode::Insert.to_string(), "INSERT");
    /// assert_eq!(EditorMode::Normal.to_string(), "NORMAL");
    /// assert_eq!(EditorMode::Prompt.to_string(), "PROMPT");
    /// ```
    pub fn to_string(&self) -> &'static str {
        match self {
            EditorMode::Insert => "INSERT",
            EditorMode::Normal => "NORMAL",
            EditorMode::Prompt => "PROMPT",
        }
    }
}

impl Default for EditorMode {
    /// Default mode is Insert
    fn default() -> Self {
        EditorMode::Insert
    }
}
