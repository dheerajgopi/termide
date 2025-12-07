//! Editor mode enumeration

use std::fmt;
use std::str::FromStr;

/// Error returned when parsing an invalid mode string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseModeError {
    /// The invalid mode string that was provided
    pub invalid: String,
}

impl fmt::Display for ParseModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown mode '{}'. Valid modes are: 'insert', 'normal', 'prompt'",
            self.invalid
        )
    }
}

impl std::error::Error for ParseModeError {}

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

impl FromStr for EditorMode {
    type Err = ParseModeError;

    /// Parse a mode string (case-insensitive with whitespace trimming)
    ///
    /// # Valid mode names
    ///
    /// - `"insert"` → `EditorMode::Insert`
    /// - `"normal"` → `EditorMode::Normal`
    /// - `"prompt"` → `EditorMode::Prompt`
    ///
    /// Parsing is case-insensitive and leading/trailing whitespace is trimmed.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::editor::EditorMode;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(EditorMode::from_str("insert").unwrap(), EditorMode::Insert);
    /// assert_eq!(EditorMode::from_str("NORMAL").unwrap(), EditorMode::Normal);
    /// assert_eq!(EditorMode::from_str("  prompt  ").unwrap(), EditorMode::Prompt);
    ///
    /// // Invalid mode names return an error
    /// assert!(EditorMode::from_str("invalid").is_err());
    /// assert!(EditorMode::from_str("").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "insert" => Ok(EditorMode::Insert),
            "normal" => Ok(EditorMode::Normal),
            "prompt" => Ok(EditorMode::Prompt),
            _ => Err(ParseModeError {
                invalid: s.to_string(),
            }),
        }
    }
}
