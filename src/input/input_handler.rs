//! High-level input handler for processing keyboard events
//!
//! This module provides the `InputHandler` struct, which wraps `KeyBindingRegistry`
//! and provides a clean, high-level API for the main event loop. It converts
//! Crossterm events to internal types and manages mode-related state changes.
//!
//! # Key Features
//!
//! - **Event Processing**: Converts Crossterm `KeyEvent` to `KeyPattern` and processes matches
//! - **Match Results**: Returns clear match outcomes (Matched, Partial, NoMatch)
//! - **Mode Awareness**: Clears sequence buffers on mode changes to prevent stale state
//! - **Registration API**: Provides methods to register bindings during initialization
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use termide::input::input_handler::{InputHandler, MatchResult};
//! use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
//! use termide::input::EditorCommand;
//! use termide::editor::EditorMode;
//! use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
//!
//! let mut handler = InputHandler::new();
//!
//! // Register a binding for 'i' to enter insert mode
//! let binding = KeyBinding::new(
//!     KeySequence::new(vec![
//!         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
//!     ]).expect("i is valid"),
//!     EditorCommand::ChangeMode(EditorMode::Insert),
//!     BindingContext::Mode(EditorMode::Normal),
//!     Priority::Default,
//! );
//! handler.register_binding(binding).expect("registration should succeed");
//!
//! // Process a key event
//! let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
//! let result = handler.process_key_event(event, EditorMode::Normal);
//!
//! match result {
//!     MatchResult::Matched(cmd) => {
//!         // Execute command
//!         assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Insert));
//!     }
//!     MatchResult::Partial => {
//!         // Wait for more keys
//!     }
//!     MatchResult::NoMatch => {
//!         // Fall back to default behavior
//!     }
//! }
//! ```
//!
//! ## Multi-key Sequences
//!
//! ```
//! use termide::input::input_handler::{InputHandler, MatchResult};
//! use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
//! use termide::input::EditorCommand;
//! use termide::editor::EditorMode;
//! use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
//!
//! let mut handler = InputHandler::new();
//!
//! // Register "dd" for delete line
//! let dd_binding = KeyBinding::new(
//!     KeySequence::new(vec![
//!         KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
//!         KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
//!     ]).expect("dd is valid"),
//!     EditorCommand::DeleteChar,
//!     BindingContext::Mode(EditorMode::Normal),
//!     Priority::Default,
//! );
//! handler.register_binding(dd_binding).unwrap();
//!
//! // First 'd' - partial match
//! let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
//! let result = handler.process_key_event(event, EditorMode::Normal);
//! assert!(matches!(result, MatchResult::Partial));
//!
//! // Second 'd' - complete match
//! let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
//! let result = handler.process_key_event(event, EditorMode::Normal);
//! assert!(matches!(result, MatchResult::Matched(_)));
//! ```
//!
//! ## Mode Change Handling
//!
//! ```
//! use termide::input::input_handler::{InputHandler, MatchResult};
//! use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
//! use termide::input::EditorCommand;
//! use termide::editor::EditorMode;
//! use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
//!
//! let mut handler = InputHandler::new();
//!
//! // Register "dd" sequence
//! let dd_binding = KeyBinding::new(
//!     KeySequence::new(vec![
//!         KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
//!         KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
//!     ]).expect("dd is valid"),
//!     EditorCommand::DeleteChar,
//!     BindingContext::Mode(EditorMode::Normal),
//!     Priority::Default,
//! );
//! handler.register_binding(dd_binding).unwrap();
//!
//! // Type 'd' in Normal mode - partial match
//! let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
//! let result = handler.process_key_event(event, EditorMode::Normal);
//! assert!(matches!(result, MatchResult::Partial));
//!
//! // Switch to Insert mode - buffer should be cleared
//! handler.on_mode_change();
//!
//! // Type 'd' again in Insert mode - should be NoMatch, not continuing the sequence
//! let result = handler.process_key_event(event, EditorMode::Insert);
//! assert!(matches!(result, MatchResult::NoMatch));
//! ```

use crate::editor::EditorMode;
use crate::input::keybinding::{KeyBinding, KeyPattern};
use crate::input::registry::{BindingError, KeyBindingRegistry};
use crate::input::EditorCommand;
use crossterm::event::KeyEvent;
use std::time::Duration;

/// Result of matching a key event against registered bindings
///
/// This enum represents the three possible outcomes when processing a key event:
/// - `Matched`: A complete binding matched, execute the command
/// - `Partial`: An incomplete multi-key sequence matched, wait for next key
/// - `NoMatch`: No binding matches, fall back to default behavior
///
/// # Examples
///
/// ```
/// use termide::input::input_handler::MatchResult;
/// use termide::input::EditorCommand;
/// use termide::editor::EditorMode;
///
/// // Complete match - execute command
/// let result = MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Insert));
/// match result {
///     MatchResult::Matched(cmd) => {
///         // Execute command
///     }
///     _ => {}
/// }
///
/// // Partial match - wait for next key
/// let result = MatchResult::Partial;
/// match result {
///     MatchResult::Partial => {
///         // Show indicator that more keys are expected
///     }
///     _ => {}
/// }
///
/// // No match - fall back to default behavior
/// let result = MatchResult::NoMatch;
/// match result {
///     MatchResult::NoMatch => {
///         // Handle key with default logic
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchResult {
    /// A complete match was found, execute this command
    ///
    /// The sequence buffer is automatically cleared when this result is returned.
    Matched(EditorCommand),

    /// An incomplete multi-key sequence matched, wait for the next key
    ///
    /// The sequence buffer is preserved to continue matching.
    Partial,

    /// No binding matches the current input
    ///
    /// The sequence buffer is automatically cleared. The main loop should
    /// fall back to default key handling behavior.
    NoMatch,
}

/// High-level input handler for processing keyboard events
///
/// `InputHandler` wraps `KeyBindingRegistry` and provides a clean API for the main
/// event loop. It handles event conversion, match result determination, and
/// mode-related state management.
///
/// # Fields
///
/// - `registry`: The underlying keybinding registry
///
/// # Examples
///
/// ```
/// use termide::input::input_handler::InputHandler;
/// use std::time::Duration;
///
/// // Create with default timeout (1000ms)
/// let handler = InputHandler::new();
///
/// // Create with custom timeout
/// let handler = InputHandler::with_timeout(Duration::from_millis(500));
/// ```
#[derive(Debug)]
pub struct InputHandler {
    /// The underlying keybinding registry
    registry: KeyBindingRegistry,
}

impl InputHandler {
    /// Creates a new input handler with default timeout (1000ms)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::input_handler::InputHandler;
    ///
    /// let handler = InputHandler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            registry: KeyBindingRegistry::default(),
        }
    }

    /// Creates a new input handler with a custom timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Duration after which incomplete sequences are cleared
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::input_handler::InputHandler;
    /// use std::time::Duration;
    ///
    /// // Create with 500ms timeout for faster feedback
    /// let handler = InputHandler::with_timeout(Duration::from_millis(500));
    ///
    /// // Create with 2 second timeout for slower typists
    /// let handler = InputHandler::with_timeout(Duration::from_secs(2));
    /// ```
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            registry: KeyBindingRegistry::new(timeout),
        }
    }

    /// Registers a new keybinding
    ///
    /// Delegates to the underlying registry. Returns an error if a binding with
    /// the same sequence, context, and priority already exists.
    ///
    /// # Arguments
    ///
    /// * `binding` - The keybinding to register
    ///
    /// # Returns
    ///
    /// - `Ok(())` if registration succeeded
    /// - `Err(BindingError)` if registration failed
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::input_handler::InputHandler;
    /// use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
    /// use termide::input::EditorCommand;
    /// use termide::editor::EditorMode;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let mut handler = InputHandler::new();
    ///
    /// let binding = KeyBinding::new(
    ///     KeySequence::new(vec![
    ///         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
    ///     ]).expect("i is valid"),
    ///     EditorCommand::ChangeMode(EditorMode::Insert),
    ///     BindingContext::Mode(EditorMode::Normal),
    ///     Priority::Default,
    /// );
    ///
    /// handler.register_binding(binding).expect("registration should succeed");
    /// ```
    pub fn register_binding(&mut self, binding: KeyBinding) -> Result<(), BindingError> {
        self.registry.register(binding)
    }

    /// Returns a mutable reference to the underlying registry
    ///
    /// This allows direct access to the registry for bulk operations like
    /// registering default bindings during initialization.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::input_handler::InputHandler;
    /// use termide::input::bindings::register_default_bindings;
    ///
    /// let mut handler = InputHandler::new();
    /// register_default_bindings(handler.registry_mut()).expect("defaults should register");
    /// ```
    pub fn registry_mut(&mut self) -> &mut KeyBindingRegistry {
        &mut self.registry
    }

    /// Processes a key event and returns the match result
    ///
    /// This method converts the Crossterm `KeyEvent` to a `KeyPattern`, adds it to
    /// the sequence buffer, and checks for matches. The sequence buffer is automatically
    /// cleared for complete matches and non-matches, but preserved for partial matches.
    ///
    /// # Arguments
    ///
    /// * `event` - The Crossterm key event to process
    /// * `mode` - The current editor mode for context filtering
    ///
    /// # Returns
    ///
    /// - `MatchResult::Matched(command)` if a complete binding matched
    /// - `MatchResult::Partial` if an incomplete sequence matched
    /// - `MatchResult::NoMatch` if no binding matches
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::input_handler::{InputHandler, MatchResult};
    /// use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
    /// use termide::input::EditorCommand;
    /// use termide::editor::EditorMode;
    /// use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
    ///
    /// let mut handler = InputHandler::new();
    ///
    /// // Register 'i' to enter insert mode
    /// let binding = KeyBinding::new(
    ///     KeySequence::new(vec![
    ///         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
    ///     ]).expect("i is valid"),
    ///     EditorCommand::ChangeMode(EditorMode::Insert),
    ///     BindingContext::Mode(EditorMode::Normal),
    ///     Priority::Default,
    /// );
    /// handler.register_binding(binding).unwrap();
    ///
    /// // Process 'i' key in Normal mode
    /// let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    /// let result = handler.process_key_event(event, EditorMode::Normal);
    ///
    /// match result {
    ///     MatchResult::Matched(cmd) => {
    ///         assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Insert));
    ///     }
    ///     _ => panic!("Expected match"),
    /// }
    /// ```
    pub fn process_key_event(&mut self, event: KeyEvent, mode: EditorMode) -> MatchResult {
        // Convert KeyEvent to KeyPattern
        let pattern = KeyPattern::new(event.code, event.modifiers);

        // Add to sequence buffer
        self.registry.add_to_sequence(pattern);

        // Check for matches
        if let Some(command) = self.registry.find_match(mode) {
            // Complete match found
            let result = MatchResult::Matched(command.clone());
            self.registry.clear_sequence();
            result
        } else if self.registry.is_partial_match(mode) {
            // Incomplete sequence, wait for next key
            MatchResult::Partial
        } else {
            // No match, clear buffer and fall back to default
            self.registry.clear_sequence();
            MatchResult::NoMatch
        }
    }

    /// Handles mode changes by clearing the sequence buffer
    ///
    /// This method should be called whenever the editor mode changes to prevent
    /// stale sequences from affecting the new mode. For example, if the user types
    /// "d" in Normal mode (starting "dd" sequence) and then switches to Insert mode,
    /// the buffer should be cleared so the next "d" isn't treated as completing the sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::input_handler::{InputHandler, MatchResult};
    /// use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
    /// use termide::input::EditorCommand;
    /// use termide::editor::EditorMode;
    /// use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
    ///
    /// let mut handler = InputHandler::new();
    ///
    /// // Register "dd" sequence
    /// let dd_binding = KeyBinding::new(
    ///     KeySequence::new(vec![
    ///         KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///         KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     ]).expect("dd is valid"),
    ///     EditorCommand::DeleteChar,
    ///     BindingContext::Mode(EditorMode::Normal),
    ///     Priority::Default,
    /// );
    /// handler.register_binding(dd_binding).unwrap();
    ///
    /// // Type 'd' - partial match
    /// let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    /// let result = handler.process_key_event(event, EditorMode::Normal);
    /// assert!(matches!(result, MatchResult::Partial));
    ///
    /// // Mode change - clear buffer
    /// handler.on_mode_change();
    ///
    /// // Type 'd' again - should not complete the sequence
    /// let result = handler.process_key_event(event, EditorMode::Insert);
    /// assert!(matches!(result, MatchResult::NoMatch));
    /// ```
    pub fn on_mode_change(&mut self) {
        self.registry.clear_sequence();
    }

    /// Checks if the sequence buffer has timed out and clears it if so
    ///
    /// This method should be called periodically (e.g., in the main event loop)
    /// to prevent incomplete sequences from keeping the editor in a waiting state
    /// indefinitely.
    ///
    /// # Returns
    ///
    /// - `true` if the buffer was cleared due to timeout
    /// - `false` if the buffer is still valid or already empty
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::input_handler::InputHandler;
    /// use std::time::Duration;
    ///
    /// let mut handler = InputHandler::with_timeout(Duration::from_millis(100));
    ///
    /// // In main event loop:
    /// loop {
    ///     // ... handle events ...
    ///
    ///     // Periodically check for timeouts
    ///     if handler.check_timeout() {
    ///         // Buffer was cleared due to timeout
    ///         // Maybe show a message to user
    ///     }
    ///
    ///     // ... render ...
    /// # break; // Exit loop for example
    /// }
    /// ```
    pub fn check_timeout(&mut self) -> bool {
        self.registry.check_timeout()
    }
}

impl Default for InputHandler {
    /// Creates a new input handler with default timeout (1000ms)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::input_handler::InputHandler;
    ///
    /// let handler = InputHandler::default();
    /// ```
    fn default() -> Self {
        Self::new()
    }
}
