//! Keybinding types for representing key patterns and key sequences
//!
//! This module provides strongly-typed representations of keyboard input patterns
//! and sequences, supporting both single-key bindings and multi-key sequences
//! (like vim's "dd" or "gg").
//!
//! # Cross-Platform Modifier Support
//!
//! The `PRIMARY_MODIFIER` constant provides cross-platform compatibility:
//! - macOS: Maps to `KeyModifiers::SUPER` (Cmd key)
//! - Linux/Windows: Maps to `KeyModifiers::CONTROL` (Ctrl key)
//!
//! This allows bindings like "Save" to use Cmd+S on macOS and Ctrl+S on other platforms.
//!
//! # Context-Aware Bindings
//!
//! Bindings can be associated with specific editor modes using `BindingContext`:
//! - `Global`: Active in all modes except Prompt
//! - `Mode(EditorMode)`: Active only in a specific mode
//! - `Modes(Vec<EditorMode>)`: Active in multiple specified modes
//! - `Plugin`: Plugin-specific bindings with optional mode filtering
//!
//! # Priority System
//!
//! When multiple bindings match the same key sequence, priority determines which wins:
//! - `Priority::User` (20): User customizations
//! - `Priority::Plugin` (10): Plugin-defined bindings
//! - `Priority::Default` (0): Built-in editor bindings
//!
//! Higher priority values take precedence.
//!
//! # Examples
//!
//! ## Single-key pattern
//!
//! ```
//! use termide::input::keybinding::{KeyPattern, PRIMARY_MODIFIER};
//! use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
//!
//! // Cross-platform save command (Cmd+S on macOS, Ctrl+S elsewhere)
//! let save_pattern = KeyPattern::new(KeyCode::Char('s'), PRIMARY_MODIFIER);
//!
//! // Match an event
//! let event = KeyEvent::new(KeyCode::Char('s'), PRIMARY_MODIFIER);
//! assert!(save_pattern.matches(&event));
//! ```
//!
//! ## Multi-key sequence
//!
//! ```
//! use termide::input::keybinding::{KeyPattern, KeySequence};
//! use crossterm::event::{KeyCode, KeyModifiers};
//!
//! // Vim-style "dd" delete line command
//! let dd_sequence = KeySequence::new(vec![
//!     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
//!     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
//! ]).expect("dd sequence is valid");
//!
//! // Check partial match (user typed first 'd')
//! let buffer = vec![KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE)];
//! assert!(dd_sequence.is_partial_match(&buffer));
//!
//! // Check complete match (user typed "dd")
//! let buffer = vec![
//!     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
//!     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
//! ];
//! assert!(dd_sequence.matches(&buffer));
//! ```
//!
//! ## Context-aware binding
//!
//! ```
//! use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
//! use termide::input::EditorCommand;
//! use termide::editor::EditorMode;
//! use crossterm::event::{KeyCode, KeyModifiers};
//!
//! // "i" to enter insert mode - only active in Normal mode
//! let insert_binding = KeyBinding::new(
//!     KeySequence::new(vec![
//!         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
//!     ]).expect("i is valid"),
//!     EditorCommand::ChangeMode(EditorMode::Insert),
//!     BindingContext::Mode(EditorMode::Normal),
//!     Priority::Default,
//! );
//!
//! // Check if binding is active in Normal mode
//! assert!(insert_binding.context().is_active(EditorMode::Normal));
//! assert!(!insert_binding.context().is_active(EditorMode::Insert));
//! ```

use crate::editor::EditorMode;
use crate::input::EditorCommand;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::str::FromStr;
use thiserror::Error;

/// Platform-specific primary modifier key
///
/// - macOS: `SUPER` (Cmd key) for native shortcuts like Cmd+S, Cmd+Q
/// - Linux/Windows: `CONTROL` (Ctrl key) for shortcuts like Ctrl+S, Ctrl+Q
///
/// This constant is resolved at compile time using conditional compilation,
/// providing zero runtime overhead while ensuring platform-appropriate behavior.
///
/// # Examples
///
/// ```
/// use termide::input::keybinding::{KeyPattern, PRIMARY_MODIFIER};
/// use crossterm::event::KeyCode;
///
/// // This binding will use Cmd+S on macOS, Ctrl+S on Linux/Windows
/// let save = KeyPattern::new(KeyCode::Char('s'), PRIMARY_MODIFIER);
/// ```
#[cfg(target_os = "macos")]
pub const PRIMARY_MODIFIER: KeyModifiers = KeyModifiers::SUPER;

#[cfg(not(target_os = "macos"))]
pub const PRIMARY_MODIFIER: KeyModifiers = KeyModifiers::CONTROL;

/// A single key pattern with required modifiers
///
/// Represents a single key press with exact modifier requirements. Modifier
/// matching is exact: Ctrl matches only Ctrl, not Ctrl+Shift.
///
/// This exact matching is important for vim-style editors where:
/// - `d` (no modifiers) is different from `D` (which is `Shift+d`)
/// - `Ctrl+S` should not trigger if `Ctrl+Shift+S` is pressed
///
/// # Fields
///
/// - `code`: The key code (character, function key, arrow, etc.)
/// - `modifiers`: Required modifiers (NONE, CONTROL, SHIFT, ALT, SUPER, or combinations)
///
/// # Examples
///
/// ```
/// use termide::input::keybinding::KeyPattern;
/// use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
///
/// // Pattern for lowercase 'i' with no modifiers
/// let insert_key = KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE);
///
/// // Pattern for Ctrl+X
/// let cut = KeyPattern::new(KeyCode::Char('x'), KeyModifiers::CONTROL);
///
/// // Pattern for Shift+D (uppercase D in vim)
/// let delete_to_end = KeyPattern::new(KeyCode::Char('d'), KeyModifiers::SHIFT);
///
/// // Matching is exact
/// let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
/// assert!(insert_key.matches(&event));
///
/// let event_with_shift = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::SHIFT);
/// assert!(!insert_key.matches(&event_with_shift)); // Different modifier
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyPattern {
    /// The key code to match
    pub code: KeyCode,
    /// Required modifiers (must match exactly)
    pub modifiers: KeyModifiers,
}

impl KeyPattern {
    /// Creates a new key pattern with the specified key code and modifiers
    ///
    /// # Arguments
    ///
    /// * `code` - The key code to match
    /// * `modifiers` - Required modifiers (must match exactly)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::keybinding::KeyPattern;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let escape = KeyPattern::new(KeyCode::Esc, KeyModifiers::NONE);
    /// let save = KeyPattern::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    /// let cut = KeyPattern::new(KeyCode::Char('x'), KeyModifiers::CONTROL | KeyModifiers::SHIFT);
    /// ```
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Checks if this pattern matches a key event with exact modifier matching
    ///
    /// Returns `true` only if both the key code and modifiers match exactly.
    /// Modifier matching is strict: Ctrl matches only Ctrl, not Ctrl+Shift.
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to match against
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::keybinding::KeyPattern;
    /// use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
    ///
    /// let pattern = KeyPattern::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    ///
    /// // Exact match
    /// let event1 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    /// assert!(pattern.matches(&event1));
    ///
    /// // Different key code
    /// let event2 = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
    /// assert!(!pattern.matches(&event2));
    ///
    /// // Different modifiers (exact matching)
    /// let event3 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL | KeyModifiers::SHIFT);
    /// assert!(!pattern.matches(&event3));
    ///
    /// // No modifiers when pattern requires them
    /// let event4 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    /// assert!(!pattern.matches(&event4));
    /// ```
    pub fn matches(&self, event: &KeyEvent) -> bool {
        self.code == event.code && self.modifiers == event.modifiers
    }
}

/// A sequence of key patterns for multi-key bindings
///
/// Represents a sequence of keys that must be pressed in order, such as
/// vim's "dd" (delete line) or "gg" (go to top).
///
/// Supports both exact matching (entire sequence typed) and partial matching
/// (user has typed a prefix of the sequence and may continue).
///
/// # Examples
///
/// ```
/// use termide::input::keybinding::{KeyPattern, KeySequence};
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// // Vim-style "dd" delete line
/// let dd = KeySequence::new(vec![
///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
/// ]).expect("dd is valid");
///
/// // Vim-style "gg" go to top
/// let gg = KeySequence::new(vec![
///     KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
///     KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
/// ]).expect("gg is valid");
///
/// // Check partial match (user typed "d")
/// let buffer = vec![KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE)];
/// assert!(dd.is_partial_match(&buffer));
/// assert!(!dd.matches(&buffer)); // Not a complete match yet
///
/// // Check complete match (user typed "dd")
/// let buffer = vec![
///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
/// ];
/// assert!(dd.matches(&buffer));
/// assert!(!dd.is_partial_match(&buffer)); // Complete, not partial
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySequence {
    /// The ordered sequence of key patterns
    patterns: Vec<KeyPattern>,
}

impl KeySequence {
    /// Creates a new key sequence from a vector of patterns
    ///
    /// Returns `None` if the patterns vector is empty, as a sequence must contain
    /// at least one key pattern.
    ///
    /// # Arguments
    ///
    /// * `patterns` - The ordered sequence of key patterns (must not be empty)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::keybinding::{KeyPattern, KeySequence};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// // Using expect() when you know the input is valid
    /// let insert = KeySequence::new(vec![
    ///     KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
    /// ]).expect("insert sequence is valid");
    ///
    /// // Multi-key sequence
    /// let delete_line = KeySequence::new(vec![
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    /// ]).expect("dd sequence is valid");
    ///
    /// // Proper error handling for dynamic input
    /// let patterns = vec![]; // Empty from user input
    /// match KeySequence::new(patterns) {
    ///     Some(seq) => { /* use sequence */ },
    ///     None => { /* handle invalid input */ }
    /// }
    /// ```
    pub fn new(patterns: Vec<KeyPattern>) -> Option<Self> {
        if patterns.is_empty() {
            None
        } else {
            Some(Self { patterns })
        }
    }

    /// Returns the number of patterns in this sequence
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::keybinding::{KeyPattern, KeySequence};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let single = KeySequence::new(vec![
    ///     KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
    /// ]).expect("single is valid");
    /// assert_eq!(single.len(), 1);
    ///
    /// let double = KeySequence::new(vec![
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    /// ]).expect("double is valid");
    /// assert_eq!(double.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Returns `true` if the sequence contains no patterns
    ///
    /// Note: Due to the invariant enforced by `new()`, this will always return `false`
    /// for sequences created through the public API.
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Checks if this sequence exactly matches the given buffer
    ///
    /// Returns `true` only if the buffer contains the exact same patterns
    /// in the same order.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The accumulated key patterns to match against
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::keybinding::{KeyPattern, KeySequence};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let dd = KeySequence::new(vec![
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    /// ]).expect("dd is valid");
    ///
    /// // Complete match
    /// let buffer = vec![
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    /// ];
    /// assert!(dd.matches(&buffer));
    ///
    /// // Partial sequence (too short)
    /// let buffer = vec![KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE)];
    /// assert!(!dd.matches(&buffer));
    ///
    /// // Too long
    /// let buffer = vec![
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    /// ];
    /// assert!(!dd.matches(&buffer));
    ///
    /// // Wrong keys
    /// let buffer = vec![
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     KeyPattern::new(KeyCode::Char('x'), KeyModifiers::NONE),
    /// ];
    /// assert!(!dd.matches(&buffer));
    /// ```
    pub fn matches(&self, buffer: &[KeyPattern]) -> bool {
        self.patterns.len() == buffer.len() && self.patterns == buffer
    }

    /// Checks if the buffer is a partial match (incomplete sequence)
    ///
    /// Returns `true` if the buffer matches a prefix of this sequence but is not
    /// yet complete. This indicates the user is in the middle of typing a multi-key
    /// sequence and more keys are expected.
    ///
    /// Returns `false` if:
    /// - Buffer is empty
    /// - Buffer is equal to or longer than the sequence
    /// - Buffer does not match the sequence prefix
    ///
    /// # Arguments
    ///
    /// * `buffer` - The accumulated key patterns to check
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::keybinding::{KeyPattern, KeySequence};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let dd = KeySequence::new(vec![
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    /// ]).expect("dd is valid");
    ///
    /// // Partial match (typed "d", expecting another "d")
    /// let buffer = vec![KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE)];
    /// assert!(dd.is_partial_match(&buffer));
    ///
    /// // Complete match (not partial)
    /// let buffer = vec![
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    /// ];
    /// assert!(!dd.is_partial_match(&buffer));
    ///
    /// // Empty buffer (not a partial match)
    /// assert!(!dd.is_partial_match(&[]));
    ///
    /// // Wrong prefix (not a partial match)
    /// let buffer = vec![KeyPattern::new(KeyCode::Char('x'), KeyModifiers::NONE)];
    /// assert!(!dd.is_partial_match(&buffer));
    /// ```
    pub fn is_partial_match(&self, buffer: &[KeyPattern]) -> bool {
        if buffer.is_empty() || buffer.len() >= self.patterns.len() {
            return false;
        }

        // Check if buffer matches the prefix of our sequence
        self.patterns
            .iter()
            .zip(buffer.iter())
            .all(|(pattern, buf_pattern)| pattern == buf_pattern)
    }
}

/// Error type for parsing key sequences and patterns from strings
///
/// This error type provides detailed information about what went wrong during
/// parsing, including the invalid portion of the input for user debugging.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input string was empty
    #[error("empty key sequence string")]
    EmptyInput,

    /// A key pattern string was empty (e.g., "Ctrl+" with nothing after)
    #[error("empty key pattern: expected key name after modifiers")]
    EmptyPattern,

    /// An unknown modifier name was encountered
    #[error("unknown modifier '{0}': valid modifiers are Ctrl, Shift, Alt, Super")]
    UnknownModifier(String),

    /// An unknown key name was encountered
    #[error("unknown key name '{0}': check spelling or use special key names like Enter, Esc, Tab, etc.")]
    UnknownKey(String),

    /// Invalid format (e.g., multiple '+' in a row)
    #[error("invalid key pattern format '{0}': expected format like 'Ctrl+S' or 'd d'")]
    InvalidFormat(String),
}

impl FromStr for KeySequence {
    type Err = ParseError;

    /// Parses a human-readable key sequence string into a `KeySequence`
    ///
    /// # Format
    ///
    /// The parser supports two formats:
    ///
    /// 1. **Single-key sequences with modifiers**: `"Ctrl+S"`, `"Alt+F4"`, `"Shift+Tab"`
    /// 2. **Multi-key sequences**: `"d d"`, `"g g"`, `"c i ("`
    ///
    /// ## Modifier Syntax
    ///
    /// - Modifiers and keys are separated by `+`
    /// - Multiple modifiers can be combined: `"Ctrl+Shift+F"`
    /// - Modifiers are case-insensitive: `"ctrl+s"` == `"Ctrl+S"` == `"CTRL+S"`
    /// - Valid modifiers: `Ctrl`, `Shift`, `Alt`, `Super`
    ///
    /// ## Special Keys
    ///
    /// Special key names are case-insensitive:
    /// - Navigation: `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`
    /// - Editing: `Enter`, `Backspace`, `Delete`, `Tab`, `Space`, `Esc`
    /// - Function keys: `F1` through `F12`
    ///
    /// ## Multi-Key Sequences
    ///
    /// - Keys are separated by spaces: `"d d"`, `"g g"`
    /// - Each key can have its own modifiers: `"Ctrl+X k"` (Ctrl+X followed by k)
    /// - Extra whitespace is ignored
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::keybinding::KeySequence;
    /// use std::str::FromStr;
    ///
    /// // Single-key with modifier
    /// let seq = KeySequence::from_str("Ctrl+S").unwrap();
    /// assert_eq!(seq.len(), 1);
    ///
    /// // Multi-key sequence
    /// let seq = KeySequence::from_str("d d").unwrap();
    /// assert_eq!(seq.len(), 2);
    ///
    /// // Multiple modifiers
    /// let seq = KeySequence::from_str("Ctrl+Shift+F").unwrap();
    /// assert_eq!(seq.len(), 1);
    ///
    /// // Case-insensitive modifiers (but character case matters)
    /// let seq1 = KeySequence::from_str("ctrl+s").unwrap();
    /// let seq2 = KeySequence::from_str("Ctrl+s").unwrap();
    /// assert_eq!(seq1, seq2);
    ///
    /// // Special keys
    /// let seq = KeySequence::from_str("Enter").unwrap();
    /// let seq = KeySequence::from_str("Ctrl+Backspace").unwrap();
    ///
    /// // Error cases
    /// assert!(KeySequence::from_str("").is_err());
    /// assert!(KeySequence::from_str("Ctrl+").is_err());
    /// assert!(KeySequence::from_str("InvalidKey").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - The input string is empty
    /// - A key pattern is incomplete (e.g., `"Ctrl+"`)
    /// - An unknown modifier is used
    /// - An unknown key name is used
    /// - The format is invalid
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        // Split by whitespace to get individual key patterns
        let pattern_strings: Vec<&str> = trimmed.split_whitespace().collect();
        let mut patterns = Vec::new();

        for pattern_str in pattern_strings {
            let pattern = parse_key_pattern(pattern_str)?;
            patterns.push(pattern);
        }

        KeySequence::new(patterns).ok_or(ParseError::EmptyInput)
    }
}

/// Parses a single key pattern string like "Ctrl+S" or "Enter"
fn parse_key_pattern(s: &str) -> Result<KeyPattern, ParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ParseError::EmptyPattern);
    }

    // Split by '+' to separate modifiers from the key
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();

    if parts.is_empty() || parts.iter().any(|p| p.is_empty()) {
        return Err(ParseError::InvalidFormat(s.to_string()));
    }

    // Last part is the key, everything before is modifiers
    let key_str = parts[parts.len() - 1];
    let modifier_strs = &parts[..parts.len() - 1];

    // Parse modifiers
    let mut modifiers = KeyModifiers::NONE;
    for modifier_str in modifier_strs {
        let modifier = parse_modifier(modifier_str)?;
        modifiers |= modifier;
    }

    // Parse key
    let key_code = parse_key_code(key_str)?;

    Ok(KeyPattern::new(key_code, modifiers))
}

/// Parses a modifier string like "Ctrl", "Shift", "Alt"
fn parse_modifier(s: &str) -> Result<KeyModifiers, ParseError> {
    match s.to_lowercase().as_str() {
        "ctrl" | "control" => Ok(KeyModifiers::CONTROL),
        "shift" => Ok(KeyModifiers::SHIFT),
        "alt" => Ok(KeyModifiers::ALT),
        "super" | "cmd" | "command" => Ok(KeyModifiers::SUPER),
        _ => Err(ParseError::UnknownModifier(s.to_string())),
    }
}

/// Parses a key name string like "s", "Enter", "F1"
fn parse_key_code(s: &str) -> Result<KeyCode, ParseError> {
    // Special keys (case-insensitive)
    match s.to_lowercase().as_str() {
        "enter" | "return" => return Ok(KeyCode::Enter),
        "esc" | "escape" => return Ok(KeyCode::Esc),
        "tab" => return Ok(KeyCode::Tab),
        "backspace" | "back" => return Ok(KeyCode::Backspace),
        "delete" | "del" => return Ok(KeyCode::Delete),
        "space" => return Ok(KeyCode::Char(' ')),
        "up" => return Ok(KeyCode::Up),
        "down" => return Ok(KeyCode::Down),
        "left" => return Ok(KeyCode::Left),
        "right" => return Ok(KeyCode::Right),
        "home" => return Ok(KeyCode::Home),
        "end" => return Ok(KeyCode::End),
        "pageup" | "pgup" => return Ok(KeyCode::PageUp),
        "pagedown" | "pgdown" => return Ok(KeyCode::PageDown),
        _ => {}
    }

    // Function keys
    if s.to_lowercase().starts_with('f') {
        if let Ok(num) = s[1..].parse::<u8>() {
            if (1..=12).contains(&num) {
                return Ok(KeyCode::F(num));
            }
        }
    }

    // Single character keys
    if s.len() == 1 {
        let ch = s.chars().next().unwrap();
        return Ok(KeyCode::Char(ch));
    }

    // Unknown key
    Err(ParseError::UnknownKey(s.to_string()))
}

/// Context specifying when a keybinding is active
///
/// Bindings can be global (active in most modes), mode-specific, or plugin-specific.
/// This allows the same key sequence to trigger different commands in different contexts.
///
/// # Variants
///
/// - `Global`: Active in Insert and Normal modes (excludes Prompt mode)
/// - `Mode(EditorMode)`: Active only in the specified mode
/// - `Modes(Vec<EditorMode>)`: Active in any of the specified modes
/// - `Plugin`: Plugin-specific bindings with optional mode filtering
///
/// # Examples
///
/// ```
/// use termide::input::keybinding::BindingContext;
/// use termide::editor::EditorMode;
///
/// // Global binding - active in Insert and Normal, not in Prompt
/// let global = BindingContext::Global;
/// assert!(global.is_active(EditorMode::Normal));
/// assert!(global.is_active(EditorMode::Insert));
/// assert!(!global.is_active(EditorMode::Prompt)); // Prompt mode excluded
///
/// // Mode-specific binding
/// let normal_only = BindingContext::Mode(EditorMode::Normal);
/// assert!(normal_only.is_active(EditorMode::Normal));
/// assert!(!normal_only.is_active(EditorMode::Insert));
///
/// // Multi-mode binding
/// let editing_modes = BindingContext::Modes(vec![
///     EditorMode::Insert,
///     EditorMode::Normal,
/// ]);
/// assert!(editing_modes.is_active(EditorMode::Insert));
/// assert!(editing_modes.is_active(EditorMode::Normal));
/// assert!(!editing_modes.is_active(EditorMode::Prompt));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingContext {
    /// Active in all modes except Prompt
    ///
    /// Global bindings are for universal shortcuts like Ctrl+S (save) that should
    /// work in both Insert and Normal modes, but not when prompting for input.
    Global,

    /// Active only in the specified mode
    ///
    /// Use this for mode-specific commands like "i" to enter Insert mode (only
    /// makes sense in Normal mode).
    Mode(EditorMode),

    /// Active in any of the specified modes
    ///
    /// Use this when a binding should work in multiple specific modes but not all.
    Modes(Vec<EditorMode>),

    /// Plugin-specific binding with optional mode filtering
    ///
    /// Plugins can register bindings with an optional list of modes. If modes is
    /// `None`, the plugin binding is active in all modes. If modes is `Some(vec)`,
    /// it's only active in those modes.
    Plugin {
        /// Name of the plugin registering this binding
        name: String,
        /// Optional mode filter - None means all modes, Some(vec) means specific modes
        modes: Option<Vec<EditorMode>>,
    },
}

impl BindingContext {
    /// Checks if this context is active in the given editor mode
    ///
    /// # Arguments
    ///
    /// * `current_mode` - The current editor mode to check against
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::keybinding::BindingContext;
    /// use termide::editor::EditorMode;
    ///
    /// // Global context excludes Prompt mode
    /// let ctx = BindingContext::Global;
    /// assert!(ctx.is_active(EditorMode::Normal));
    /// assert!(ctx.is_active(EditorMode::Insert));
    /// assert!(!ctx.is_active(EditorMode::Prompt));
    ///
    /// // Mode-specific context
    /// let ctx = BindingContext::Mode(EditorMode::Normal);
    /// assert!(ctx.is_active(EditorMode::Normal));
    /// assert!(!ctx.is_active(EditorMode::Insert));
    ///
    /// // Multi-mode context
    /// let ctx = BindingContext::Modes(vec![EditorMode::Insert, EditorMode::Normal]);
    /// assert!(ctx.is_active(EditorMode::Insert));
    /// assert!(ctx.is_active(EditorMode::Normal));
    /// assert!(!ctx.is_active(EditorMode::Prompt));
    ///
    /// // Plugin context without mode filter (all modes)
    /// let ctx = BindingContext::Plugin {
    ///     name: "lsp".to_string(),
    ///     modes: None,
    /// };
    /// assert!(ctx.is_active(EditorMode::Normal));
    /// assert!(ctx.is_active(EditorMode::Insert));
    /// assert!(ctx.is_active(EditorMode::Prompt));
    ///
    /// // Plugin context with mode filter
    /// let ctx = BindingContext::Plugin {
    ///     name: "lsp".to_string(),
    ///     modes: Some(vec![EditorMode::Normal, EditorMode::Insert]),
    /// };
    /// assert!(ctx.is_active(EditorMode::Normal));
    /// assert!(ctx.is_active(EditorMode::Insert));
    /// assert!(!ctx.is_active(EditorMode::Prompt));
    /// ```
    pub fn is_active(&self, current_mode: EditorMode) -> bool {
        match self {
            BindingContext::Global => {
                // Global bindings exclude Prompt mode
                current_mode != EditorMode::Prompt
            }
            BindingContext::Mode(mode) => *mode == current_mode,
            BindingContext::Modes(modes) => modes.contains(&current_mode),
            BindingContext::Plugin { modes, .. } => match modes {
                None => true, // Active in all modes
                Some(mode_list) => mode_list.contains(&current_mode),
            },
        }
    }
}

/// Priority for keybinding conflict resolution
///
/// When multiple bindings match the same key sequence in the same context,
/// the binding with the highest priority wins. This allows user customizations
/// to override plugin defaults, and plugins to override editor defaults.
///
/// # Priority Levels
///
/// - `Default` (0): Built-in editor bindings
/// - `Plugin` (10): Plugin-defined bindings
/// - `User` (20): User customizations
///
/// Higher numeric values have higher priority.
///
/// # Examples
///
/// ```
/// use termide::input::keybinding::Priority;
///
/// // Priority ordering
/// assert!(Priority::User > Priority::Plugin);
/// assert!(Priority::Plugin > Priority::Default);
///
/// // Numeric values
/// assert_eq!(Priority::Default as u8, 0);
/// assert_eq!(Priority::Plugin as u8, 10);
/// assert_eq!(Priority::User as u8, 20);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Default editor bindings (priority 0)
    Default = 0,
    /// Plugin-defined bindings (priority 10)
    Plugin = 10,
    /// User customizations (priority 20)
    User = 20,
}

/// A complete keybinding that maps a sequence to a command in a specific context
///
/// This struct combines all the pieces needed for a keybinding:
/// - `sequence`: The key sequence to match
/// - `command`: The command to execute when matched
/// - `context`: When this binding is active (mode-dependent)
/// - `priority`: Used to resolve conflicts when multiple bindings match
///
/// # Examples
///
/// ```
/// use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
/// use termide::input::EditorCommand;
/// use termide::editor::EditorMode;
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// // Create a binding for "i" to enter insert mode (Normal mode only)
/// let insert_binding = KeyBinding::new(
///     KeySequence::new(vec![
///         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
///     ]).expect("i is valid"),
///     EditorCommand::ChangeMode(EditorMode::Insert),
///     BindingContext::Mode(EditorMode::Normal),
///     Priority::Default,
/// );
///
/// // Access binding properties
/// assert_eq!(insert_binding.priority(), Priority::Default);
/// assert!(insert_binding.context().is_active(EditorMode::Normal));
/// assert!(!insert_binding.context().is_active(EditorMode::Insert));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct KeyBinding {
    /// The key sequence to match
    sequence: KeySequence,
    /// The command to execute when matched
    command: EditorCommand,
    /// The context in which this binding is active
    context: BindingContext,
    /// Priority for conflict resolution
    priority: Priority,
}

impl KeyBinding {
    /// Creates a new keybinding
    ///
    /// # Arguments
    ///
    /// * `sequence` - The key sequence to match
    /// * `command` - The command to execute when matched
    /// * `context` - The context in which this binding is active
    /// * `priority` - Priority for conflict resolution
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
    /// use termide::input::EditorCommand;
    /// use termide::editor::EditorMode;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// // Global save binding
    /// let save = KeyBinding::new(
    ///     KeySequence::new(vec![
    ///         KeyPattern::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
    ///     ]).expect("Ctrl+S is valid"),
    ///     EditorCommand::Save,
    ///     BindingContext::Global,
    ///     Priority::Default,
    /// );
    ///
    /// // Mode-specific binding
    /// let insert = KeyBinding::new(
    ///     KeySequence::new(vec![
    ///         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
    ///     ]).expect("i is valid"),
    ///     EditorCommand::ChangeMode(EditorMode::Insert),
    ///     BindingContext::Mode(EditorMode::Normal),
    ///     Priority::Default,
    /// );
    /// ```
    pub fn new(
        sequence: KeySequence,
        command: EditorCommand,
        context: BindingContext,
        priority: Priority,
    ) -> Self {
        Self {
            sequence,
            command,
            context,
            priority,
        }
    }

    /// Returns a reference to the key sequence
    pub fn sequence(&self) -> &KeySequence {
        &self.sequence
    }

    /// Returns a reference to the command
    pub fn command(&self) -> &EditorCommand {
        &self.command
    }

    /// Returns a reference to the binding context
    pub fn context(&self) -> &BindingContext {
        &self.context
    }

    /// Returns the priority
    pub fn priority(&self) -> Priority {
        self.priority
    }
}
