//! Central keybinding registry for storing and managing keybindings
//!
//! This module provides the `KeyBindingRegistry` that stores, manages, and retrieves
//! keybindings with priority-based conflict resolution. The registry maintains bindings
//! in priority-sorted order for efficient lookup and handles duplicate detection.
//!
//! # Key Features
//!
//! - **Priority-Sorted Storage**: Bindings are stored in descending priority order
//! - **Conflict Detection**: Same sequence + context + priority = error
//! - **Override Support**: Higher priority bindings can shadow lower priority ones
//! - **Dynamic Management**: Bindings can be registered and unregistered at runtime
//! - **Timeout Support**: Configurable timeout for multi-key sequences
//!
//! # Examples
//!
//! ## Basic Registration
//!
//! ```
//! use termide::input::registry::KeyBindingRegistry;
//! use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
//! use termide::input::EditorCommand;
//! use termide::editor::EditorMode;
//! use crossterm::event::{KeyCode, KeyModifiers};
//! use std::time::Duration;
//!
//! let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
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
//!
//! registry.register(binding).expect("registration should succeed");
//! ```
//!
//! ## Conflict Detection
//!
//! ```
//! use termide::input::registry::{KeyBindingRegistry, BindingError};
//! use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
//! use termide::input::EditorCommand;
//! use termide::editor::EditorMode;
//! use crossterm::event::{KeyCode, KeyModifiers};
//! use std::time::Duration;
//!
//! let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
//!
//! let binding1 = KeyBinding::new(
//!     KeySequence::new(vec![
//!         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
//!     ]).expect("i is valid"),
//!     EditorCommand::ChangeMode(EditorMode::Insert),
//!     BindingContext::Mode(EditorMode::Normal),
//!     Priority::Default,
//! );
//!
//! let binding2 = KeyBinding::new(
//!     KeySequence::new(vec![
//!         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
//!     ]).expect("i is valid"),
//!     EditorCommand::ChangeMode(EditorMode::Insert), // Different command, same sequence+context+priority
//!     BindingContext::Mode(EditorMode::Normal),
//!     Priority::Default,
//! );
//!
//! registry.register(binding1).expect("first registration should succeed");
//! let result = registry.register(binding2);
//! assert!(matches!(result, Err(BindingError::Conflict { .. })));
//! ```
//!
//! ## Priority Override
//!
//! ```
//! use termide::input::registry::KeyBindingRegistry;
//! use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
//! use termide::input::EditorCommand;
//! use termide::editor::EditorMode;
//! use crossterm::event::{KeyCode, KeyModifiers};
//! use std::time::Duration;
//!
//! let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
//!
//! // Register default binding
//! let default_binding = KeyBinding::new(
//!     KeySequence::new(vec![
//!         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
//!     ]).expect("i is valid"),
//!     EditorCommand::ChangeMode(EditorMode::Insert),
//!     BindingContext::Mode(EditorMode::Normal),
//!     Priority::Default,
//! );
//!
//! // Register user override (higher priority)
//! let user_binding = KeyBinding::new(
//!     KeySequence::new(vec![
//!         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
//!     ]).expect("i is valid"),
//!     EditorCommand::ChangeMode(EditorMode::Insert),
//!     BindingContext::Mode(EditorMode::Normal),
//!     Priority::User, // Higher priority
//! );
//!
//! registry.register(default_binding).expect("default should succeed");
//! registry.register(user_binding).expect("user override should succeed");
//! // User binding shadows the default binding
//! ```

use crate::editor::EditorMode;
use crate::input::keybinding::{BindingContext, KeyBinding, KeyPattern, KeySequence, Priority};
use crate::input::EditorCommand;
use std::str::FromStr;
use std::time::{Duration, Instant};
use termide_plugin_api::input::{PluginBinding, PluginInputExtension};
use thiserror::Error;

/// Errors that can occur during keybinding registration or management
#[derive(Debug, Error, PartialEq)]
pub enum BindingError {
    /// A binding with the same sequence, context, and priority already exists
    ///
    /// This prevents accidental duplicate registrations at the same priority level.
    /// To override an existing binding, use a higher priority level.
    #[error("Binding conflict: sequence {sequence:?} with context {context:?} at priority {priority:?} already exists")]
    Conflict {
        sequence: String,
        context: String,
        priority: Priority,
    },

    /// The provided key sequence is invalid (empty)
    #[error("Invalid key sequence: {0}")]
    InvalidSequence(String),

    /// The provided binding context is invalid
    #[error("Invalid binding context: {0}")]
    InvalidContext(String),
}

/// Central registry for storing and managing keybindings
///
/// The registry maintains bindings in priority-sorted order (descending) for efficient
/// lookup and conflict resolution. Bindings with higher priority values take precedence
/// over lower priority bindings when the same sequence is registered in the same context.
///
/// # Fields
///
/// - `bindings`: Vector of bindings sorted by priority (descending)
/// - `sequence_buffer`: Accumulated key patterns for multi-key sequence detection
/// - `last_key_time`: Timestamp of the last key added to the buffer
/// - `timeout`: Duration after which the sequence buffer is automatically cleared
///
/// # Examples
///
/// ```
/// use termide::input::registry::KeyBindingRegistry;
/// use std::time::Duration;
///
/// // Create registry with 1 second timeout for multi-key sequences
/// let registry = KeyBindingRegistry::new(Duration::from_secs(1));
/// ```
#[derive(Debug)]
pub struct KeyBindingRegistry {
    /// Bindings sorted by priority (descending)
    bindings: Vec<KeyBinding>,
    /// Accumulated key patterns for sequence matching
    sequence_buffer: Vec<KeyPattern>,
    /// Timestamp of last key press
    last_key_time: Instant,
    /// Timeout duration for sequence completion
    timeout: Duration,
}

impl KeyBindingRegistry {
    /// Creates a new empty registry with the specified timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Duration after which incomplete sequences are cleared
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use std::time::Duration;
    ///
    /// // Create with 1 second timeout
    /// let registry = KeyBindingRegistry::new(Duration::from_secs(1));
    ///
    /// // Create with 500ms timeout for faster feedback
    /// let registry = KeyBindingRegistry::new(Duration::from_millis(500));
    /// ```
    pub fn new(timeout: Duration) -> Self {
        Self {
            bindings: Vec::new(),
            sequence_buffer: Vec::new(),
            last_key_time: Instant::now(),
            timeout,
        }
    }

    /// Registers a new keybinding in the registry
    ///
    /// Bindings are inserted in priority-sorted order (descending). If a binding with
    /// the same sequence, context, and priority already exists, returns an error.
    /// Higher priority bindings can shadow lower priority ones without conflict.
    ///
    /// # Arguments
    ///
    /// * `binding` - The keybinding to register
    ///
    /// # Returns
    ///
    /// - `Ok(())` if registration succeeded
    /// - `Err(BindingError::Conflict)` if a binding with same sequence, context, and priority exists
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
    /// use termide::input::EditorCommand;
    /// use termide::editor::EditorMode;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    /// use std::time::Duration;
    ///
    /// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
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
    /// registry.register(binding).expect("registration should succeed");
    /// ```
    pub fn register(&mut self, binding: KeyBinding) -> Result<(), BindingError> {
        // Check for conflicts: same sequence + same context + same priority
        let has_conflict = self.bindings.iter().any(|existing| {
            existing.sequence() == binding.sequence()
                && existing.context() == binding.context()
                && existing.priority() == binding.priority()
        });

        if has_conflict {
            return Err(BindingError::Conflict {
                sequence: format!("{:?}", binding.sequence()),
                context: format!("{:?}", binding.context()),
                priority: binding.priority(),
            });
        }

        // Find the correct position to insert based on priority (descending order)
        let insert_pos = self
            .bindings
            .binary_search_by(|existing| existing.priority().cmp(&binding.priority()).reverse())
            .unwrap_or_else(|pos| pos);

        self.bindings.insert(insert_pos, binding);
        Ok(())
    }

    /// Unregisters a keybinding from the registry
    ///
    /// Removes the binding that matches both the sequence and context. If no such
    /// binding exists, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `sequence` - The key sequence to match
    /// * `context` - The binding context to match
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
    /// use termide::input::EditorCommand;
    /// use termide::editor::EditorMode;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    /// use std::time::Duration;
    ///
    /// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    ///
    /// let sequence = KeySequence::new(vec![
    ///     KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
    /// ]).expect("i is valid");
    ///
    /// let binding = KeyBinding::new(
    ///     sequence.clone(),
    ///     EditorCommand::ChangeMode(EditorMode::Insert),
    ///     BindingContext::Mode(EditorMode::Normal),
    ///     Priority::Default,
    /// );
    ///
    /// registry.register(binding).expect("registration should succeed");
    /// registry.unregister(&sequence, &BindingContext::Mode(EditorMode::Normal));
    /// ```
    pub fn unregister(&mut self, sequence: &KeySequence, context: &BindingContext) {
        self.bindings.retain(|binding| {
            !(binding.sequence() == sequence && binding.context() == context)
        });
    }

    /// Unregisters all keybindings with the specified priority
    ///
    /// This is useful for removing an entire class of bindings, such as all User
    /// bindings when reloading configuration. Bindings with other priorities are
    /// preserved.
    ///
    /// # Arguments
    ///
    /// * `priority` - The priority level to remove
    ///
    /// # Returns
    ///
    /// The number of bindings that were removed
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
    /// use termide::input::EditorCommand;
    /// use termide::editor::EditorMode;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    /// use std::time::Duration;
    ///
    /// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    ///
    /// // Register some default bindings
    /// let default_binding = KeyBinding::new(
    ///     KeySequence::new(vec![
    ///         KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
    ///     ]).expect("i is valid"),
    ///     EditorCommand::ChangeMode(EditorMode::Insert),
    ///     BindingContext::Mode(EditorMode::Normal),
    ///     Priority::Default,
    /// );
    /// registry.register(default_binding).unwrap();
    ///
    /// // Register some user bindings
    /// let user_binding = KeyBinding::new(
    ///     KeySequence::new(vec![
    ///         KeyPattern::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
    ///     ]).expect("Ctrl+s is valid"),
    ///     EditorCommand::Save,
    ///     BindingContext::Global,
    ///     Priority::User,
    /// );
    /// registry.register(user_binding).unwrap();
    ///
    /// assert_eq!(registry.len(), 2);
    ///
    /// // Remove all user bindings (for config reload)
    /// let removed = registry.unregister_by_priority(Priority::User);
    /// assert_eq!(removed, 1);
    /// assert_eq!(registry.len(), 1); // Default binding remains
    /// ```
    pub fn unregister_by_priority(&mut self, priority: Priority) -> usize {
        let original_len = self.bindings.len();
        self.bindings.retain(|binding| binding.priority() != priority);
        original_len - self.bindings.len()
    }

    /// Returns the number of registered bindings
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use std::time::Duration;
    ///
    /// let registry = KeyBindingRegistry::new(Duration::from_secs(1));
    /// assert_eq!(registry.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Returns `true` if the registry contains no bindings
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use std::time::Duration;
    ///
    /// let registry = KeyBindingRegistry::new(Duration::from_secs(1));
    /// assert!(registry.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Adds a key pattern to the sequence buffer
    ///
    /// This method is called for each key press to accumulate patterns for multi-key
    /// sequence matching. It updates the timestamp to track timeout.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The key pattern to append to the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use termide::input::keybinding::KeyPattern;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    /// use std::time::Duration;
    ///
    /// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    ///
    /// // User presses 'd'
    /// let pattern = KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE);
    /// registry.add_to_sequence(pattern);
    ///
    /// // User presses 'd' again (completing "dd" sequence)
    /// registry.add_to_sequence(pattern);
    /// ```
    #[inline]
    pub fn add_to_sequence(&mut self, pattern: KeyPattern) {
        self.sequence_buffer.push(pattern);
        self.last_key_time = Instant::now();
    }

    /// Finds a matching binding for the current sequence buffer
    ///
    /// Searches for bindings that match the current buffer in the given mode.
    /// Only considers bindings where the context is active in the current mode.
    /// Returns the command from the highest-priority exact match.
    ///
    /// # Arguments
    ///
    /// * `current_mode` - The current editor mode for context filtering
    ///
    /// # Returns
    ///
    /// - `Some(&EditorCommand)` if an exact match is found
    /// - `None` if no binding matches the current buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
    /// use termide::input::EditorCommand;
    /// use termide::editor::EditorMode;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    /// use std::time::Duration;
    ///
    /// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    ///
    /// // Register "dd" for delete line
    /// let dd_binding = KeyBinding::new(
    ///     KeySequence::new(vec![
    ///         KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///         KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     ]).expect("dd is valid"),
    ///     EditorCommand::DeleteChar,
    ///     BindingContext::Mode(EditorMode::Normal),
    ///     Priority::Default,
    /// );
    /// registry.register(dd_binding).unwrap();
    ///
    /// // Add first 'd' - no match yet
    /// registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    /// assert_eq!(registry.find_match(EditorMode::Normal), None);
    ///
    /// // Add second 'd' - now we have a match
    /// registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    /// assert!(registry.find_match(EditorMode::Normal).is_some());
    /// ```
    #[inline]
    pub fn find_match(&self, current_mode: EditorMode) -> Option<&EditorCommand> {
        // Filter bindings by active context, then find first exact match
        // Since bindings are priority-sorted, first match is highest priority
        self.bindings
            .iter()
            .filter(|binding| binding.context().is_active(current_mode))
            .find(|binding| binding.sequence().matches(&self.sequence_buffer))
            .map(|binding| binding.command())
    }

    /// Checks if the current buffer partially matches any binding
    ///
    /// Returns `true` if any binding in the current mode has a sequence that starts
    /// with the current buffer but is longer (incomplete sequence). This indicates
    /// the user is in the middle of typing a multi-key sequence.
    ///
    /// # Arguments
    ///
    /// * `current_mode` - The current editor mode for context filtering
    ///
    /// # Returns
    ///
    /// - `true` if any active binding partially matches the buffer
    /// - `false` if no partial matches exist
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use termide::input::keybinding::{KeyPattern, KeySequence, KeyBinding, BindingContext, Priority};
    /// use termide::input::EditorCommand;
    /// use termide::editor::EditorMode;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    /// use std::time::Duration;
    ///
    /// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    ///
    /// // Register "dd" for delete line
    /// let dd_binding = KeyBinding::new(
    ///     KeySequence::new(vec![
    ///         KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///         KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ///     ]).expect("dd is valid"),
    ///     EditorCommand::DeleteChar,
    ///     BindingContext::Mode(EditorMode::Normal),
    ///     Priority::Default,
    /// );
    /// registry.register(dd_binding).unwrap();
    ///
    /// // Add first 'd' - this is a partial match
    /// registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    /// assert!(registry.is_partial_match(EditorMode::Normal));
    ///
    /// // Add second 'd' - no longer partial (it's complete)
    /// registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    /// assert!(!registry.is_partial_match(EditorMode::Normal));
    /// ```
    #[inline]
    pub fn is_partial_match(&self, current_mode: EditorMode) -> bool {
        self.bindings
            .iter()
            .filter(|binding| binding.context().is_active(current_mode))
            .any(|binding| binding.sequence().is_partial_match(&self.sequence_buffer))
    }

    /// Checks if the sequence buffer has timed out and clears it if so
    ///
    /// This method should be called periodically to prevent incomplete sequences
    /// from keeping the editor in a waiting state indefinitely.
    ///
    /// # Returns
    ///
    /// - `true` if the buffer was cleared due to timeout
    /// - `false` if the buffer is still valid or already empty
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use termide::input::keybinding::KeyPattern;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    /// use std::time::Duration;
    /// use std::thread;
    ///
    /// let mut registry = KeyBindingRegistry::new(Duration::from_millis(100));
    ///
    /// // Add a key
    /// registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    ///
    /// // Check immediately - no timeout
    /// assert!(!registry.check_timeout());
    ///
    /// // Wait for timeout
    /// thread::sleep(Duration::from_millis(150));
    ///
    /// // Check again - buffer should be cleared
    /// assert!(registry.check_timeout());
    ///
    /// // Second check - buffer already empty
    /// assert!(!registry.check_timeout());
    /// ```
    pub fn check_timeout(&mut self) -> bool {
        if self.sequence_buffer.is_empty() {
            return false;
        }

        let elapsed = Instant::now() - self.last_key_time;
        if elapsed >= self.timeout {
            self.sequence_buffer.clear();
            true
        } else {
            false
        }
    }

    /// Clears the sequence buffer
    ///
    /// This is typically called when a complete match is found or when switching modes.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    /// use std::time::Duration;
    ///
    /// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    /// registry.clear_sequence();
    /// ```
    pub fn clear_sequence(&mut self) {
        self.sequence_buffer.clear();
    }
}

impl Default for KeyBindingRegistry {
    /// Creates a new registry with a default timeout of 1 second
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::input::registry::KeyBindingRegistry;
    ///
    /// let registry = KeyBindingRegistry::default();
    /// ```
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

/// Implementation of PluginInputExtension for plugin keybinding registration
///
/// This implementation allows plugins to register keybindings through the
/// plugin API. It converts plugin types (string-based) to core types (strongly-typed),
/// validates them, and registers with Plugin priority.
///
/// # Examples
///
/// ```
/// use termide::input::registry::KeyBindingRegistry;
/// use termide_plugin_api::input::{PluginInputExtension, PluginBindingBuilder};
/// use std::time::Duration;
///
/// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
///
/// // Register a plugin binding
/// let binding = PluginBindingBuilder::new("my-plugin")
///     .bind("Ctrl+K", "show_info")
///     .global()
///     .build()
///     .unwrap();
///
/// registry.register_keybinding(binding).unwrap();
/// ```
impl PluginInputExtension for KeyBindingRegistry {
    fn register_keybinding(
        &mut self,
        binding: PluginBinding,
    ) -> Result<(), termide_plugin_api::input::BindingError> {
        // Convert PluginBinding (string-based) to KeyBinding (strongly-typed)

        // 1. Parse the key sequence string
        let sequence = KeySequence::from_str(&binding.sequence).map_err(|e| {
            termide_plugin_api::input::BindingError::InvalidSequence(
                binding.sequence.clone(),
                e.to_string(),
            )
        })?;

        // 2. Parse the command string
        let command = EditorCommand::from_str(&binding.command).map_err(|_e| {
            // For plugin commands that are properly namespaced, parsing should succeed
            // If it doesn't, return a generic error
            termide_plugin_api::input::BindingError::InvalidSequence(
                binding.command.clone(),
                "failed to parse command".to_string(),
            )
        })?;

        // 3. Validate plugin command namespace matches expected pattern
        // If the command is a plugin command, we can do additional validation
        if let EditorCommand::PluginCommand { plugin_name, .. } = &command {
            // Sanity check that the plugin name is not empty
            // The PluginBindingBuilder already handles command namespacing,
            // so we trust it has done its job correctly.
            if plugin_name.is_empty() {
                return Err(termide_plugin_api::input::BindingError::InvalidSequence(
                    binding.command.clone(),
                    "plugin name cannot be empty".to_string(),
                ));
            }
        }

        // 4. Convert BindingContext from plugin API to core type
        let context = convert_plugin_context_to_core(binding.context);

        // 5. Create KeyBinding with Plugin priority
        let key_binding = KeyBinding::new(sequence, command, context, Priority::Plugin);

        // 6. Register the binding with conflict detection
        self.register(key_binding).map_err(|e| match e {
            BindingError::Conflict {
                sequence,
                context,
                priority,
            } => {
                // Check if this is a Plugin priority conflict
                if priority == Priority::Plugin {
                    // Extract command info from existing binding for better error message
                    // Find the conflicting binding
                    let existing_cmd = self
                        .bindings
                        .iter()
                        .find(|b| {
                            format!("{:?}", b.sequence()) == sequence
                                && format!("{:?}", b.context()) == context
                                && b.priority() == Priority::Plugin
                        })
                        .map(|b| format!("{:?}", b.command()))
                        .unwrap_or_else(|| "unknown".to_string());

                    termide_plugin_api::input::BindingError::Conflict {
                        sequence: binding.sequence.clone(),
                        existing_command: existing_cmd,
                        plugin: "unknown".to_string(), // We don't track plugin names in core
                    }
                } else {
                    // Not a plugin conflict, but shouldn't happen since we use Plugin priority
                    termide_plugin_api::input::BindingError::InvalidSequence(
                        binding.sequence.clone(),
                        "unexpected conflict".to_string(),
                    )
                }
            }
            BindingError::InvalidSequence(msg) => {
                termide_plugin_api::input::BindingError::InvalidSequence(
                    binding.sequence.clone(),
                    msg,
                )
            }
            BindingError::InvalidContext(msg) => {
                termide_plugin_api::input::BindingError::InvalidSequence(
                    binding.sequence.clone(),
                    format!("invalid context: {}", msg),
                )
            }
        })
    }
}

/// Converts a plugin API BindingContext to the core BindingContext
///
/// This helper function maps between the plugin API types and internal core types.
fn convert_plugin_context_to_core(
    plugin_context: termide_plugin_api::input::BindingContext,
) -> BindingContext {
    match plugin_context {
        termide_plugin_api::input::BindingContext::Global => BindingContext::Global,
        termide_plugin_api::input::BindingContext::Mode(mode) => {
            BindingContext::Mode(convert_plugin_mode_to_core(mode))
        }
        termide_plugin_api::input::BindingContext::Modes(modes) => {
            let core_modes = modes
                .into_iter()
                .map(convert_plugin_mode_to_core)
                .collect();
            BindingContext::Modes(core_modes)
        }
    }
}

/// Converts a plugin API EditorMode to the core EditorMode
fn convert_plugin_mode_to_core(
    plugin_mode: termide_plugin_api::input::EditorMode,
) -> EditorMode {
    match plugin_mode {
        termide_plugin_api::input::EditorMode::Insert => EditorMode::Insert,
        termide_plugin_api::input::EditorMode::Normal => EditorMode::Normal,
        termide_plugin_api::input::EditorMode::Prompt => EditorMode::Prompt,
    }
}
