//! Input system extension API for plugins
//!
//! This module provides the interfaces for plugins to register custom keybindings.
//! Plugins can map key sequences to commands, with support for mode-specific bindings
//! and automatic command namespacing.
//!
//! # Quick Start
//!
//! ```rust
//! use termide_plugin_api::input::{PluginInputExtension, PluginBindingBuilder};
//!
//! fn setup_keybindings(registry: &mut impl PluginInputExtension) -> Result<(), Box<dyn std::error::Error>> {
//!     // Global binding (active in Insert and Normal modes)
//!     registry.register_keybinding(
//!         PluginBindingBuilder::new("my-plugin")
//!             .bind("Ctrl+K", "show_info")
//!             .global()
//!             .build()?
//!     )?;
//!
//!     // Normal mode only
//!     registry.register_keybinding(
//!         PluginBindingBuilder::new("my-plugin")
//!             .bind("g d", "goto_definition")
//!             .in_mode("normal")
//!             .build()?
//!     )?;
//!
//!     // Multiple modes
//!     registry.register_keybinding(
//!         PluginBindingBuilder::new("my-plugin")
//!             .bind("Ctrl+/", "toggle_comment")
//!             .in_modes(&["insert", "normal"])
//!             .build()?
//!     )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Key Sequence Format
//!
//! Key sequences use a human-readable string format:
//!
//! ## Single Key with Modifiers
//! - `"Ctrl+S"` - Save command
//! - `"Alt+F4"` - Close window
//! - `"Ctrl+Shift+F"` - Format code
//!
//! ## Multi-Key Sequences
//! - `"g d"` - Go to definition (vim-style)
//! - `"d d"` - Delete line
//! - `"Ctrl+X k"` - Kill buffer (emacs-style)
//!
//! ## Special Keys
//! - Navigation: `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`
//! - Editing: `Enter`, `Backspace`, `Delete`, `Tab`, `Space`, `Esc`
//! - Function: `F1` through `F12`
//!
//! ## Modifiers
//! - `Ctrl` - Control key (case-insensitive)
//! - `Shift` - Shift key
//! - `Alt` - Alt key
//! - `Super` (or `Cmd`) - Windows/Command key
//!
//! # Editor Modes
//!
//! TermIDE has three editor modes:
//! - **Insert**: Characters typed are inserted into the buffer
//! - **Normal**: Navigation and command mode (vim-style)
//! - **Prompt**: User is being prompted for input
//!
//! Bindings can be:
//! - **Global**: Active in Insert and Normal modes (not Prompt)
//! - **Mode-specific**: Active only in specified mode(s)
//!
//! # Command Namespacing
//!
//! Plugin commands are automatically namespaced with your plugin name:
//! - You specify: `"format"`
//! - Becomes: `"my-plugin.format"`
//!
//! If your command already contains a dot, it's used as-is:
//! - You specify: `"my-plugin.format"` → `"my-plugin.format"` (unchanged)

use thiserror::Error;

/// Editor mode enumeration for plugin use
///
/// Represents the editor modes that plugins can specify for keybindings.
/// This is a simplified representation for the plugin API - the actual editor
/// uses a more detailed internal representation.
///
/// # Examples
///
/// ```
/// use termide_plugin_api::input::EditorMode;
///
/// let mode = EditorMode::Insert;
/// assert_eq!(mode.as_str(), "insert");
///
/// let mode = EditorMode::from_str("normal").unwrap();
/// assert_eq!(mode, EditorMode::Normal);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorMode {
    /// Insert mode - characters are inserted at the cursor
    Insert,
    /// Normal mode - navigation and commands
    Normal,
    /// Prompt mode - user is being prompted for input
    Prompt,
}

impl EditorMode {
    /// Returns the string representation of the mode
    ///
    /// # Examples
    ///
    /// ```
    /// use termide_plugin_api::input::EditorMode;
    ///
    /// assert_eq!(EditorMode::Insert.as_str(), "insert");
    /// assert_eq!(EditorMode::Normal.as_str(), "normal");
    /// assert_eq!(EditorMode::Prompt.as_str(), "prompt");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            EditorMode::Insert => "insert",
            EditorMode::Normal => "normal",
            EditorMode::Prompt => "prompt",
        }
    }

    /// Parses a mode string into an `EditorMode`
    ///
    /// Case-insensitive and trims whitespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide_plugin_api::input::EditorMode;
    ///
    /// assert_eq!(EditorMode::from_str("insert").unwrap(), EditorMode::Insert);
    /// assert_eq!(EditorMode::from_str("Normal").unwrap(), EditorMode::Normal);
    /// assert_eq!(EditorMode::from_str(" PROMPT ").unwrap(), EditorMode::Prompt);
    /// assert!(EditorMode::from_str("invalid").is_err());
    /// ```
    pub fn from_str(s: &str) -> Result<Self, BindingError> {
        match s.trim().to_lowercase().as_str() {
            "insert" => Ok(EditorMode::Insert),
            "normal" => Ok(EditorMode::Normal),
            "prompt" => Ok(EditorMode::Prompt),
            _ => Err(BindingError::InvalidMode(s.to_string())),
        }
    }
}

/// Context specifying when a plugin keybinding is active
///
/// This determines in which editor modes the keybinding should trigger.
///
/// # Examples
///
/// ```
/// use termide_plugin_api::input::{BindingContext, EditorMode};
///
/// // Global binding (Insert and Normal, not Prompt)
/// let ctx = BindingContext::Global;
///
/// // Single mode
/// let ctx = BindingContext::Mode(EditorMode::Normal);
///
/// // Multiple modes
/// let ctx = BindingContext::Modes(vec![EditorMode::Insert, EditorMode::Normal]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingContext {
    /// Active in all modes except Prompt (Insert and Normal)
    Global,
    /// Active only in the specified mode
    Mode(EditorMode),
    /// Active in any of the specified modes
    Modes(Vec<EditorMode>),
}

/// Priority level for plugin keybindings
///
/// Plugin bindings always use `Plugin` priority (10), which is higher than
/// default editor bindings (0) but lower than user customizations (20).
///
/// This is exposed for documentation purposes - plugins cannot change their priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Priority {
    /// Default editor bindings (priority 0)
    Default = 0,
    /// Plugin-defined bindings (priority 10) - **used for all plugin bindings**
    Plugin = 10,
    /// User customizations (priority 20)
    User = 20,
}

/// Error types for plugin binding registration
///
/// Provides detailed error information when binding registration fails.
///
/// # Examples
///
/// ```
/// use termide_plugin_api::input::BindingError;
///
/// let err = BindingError::EmptySequence;
/// assert_eq!(err.to_string(), "key sequence cannot be empty");
///
/// let err = BindingError::InvalidMode("insrt".to_string());
/// assert!(err.to_string().contains("insrt"));
/// ```
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BindingError {
    /// Key sequence string is empty
    #[error("key sequence cannot be empty")]
    EmptySequence,

    /// Command string is empty
    #[error("command cannot be empty")]
    EmptyCommand,

    /// Invalid key sequence format
    #[error("invalid key sequence '{0}': {1}")]
    InvalidSequence(String, String),

    /// Invalid mode name
    #[error("invalid mode '{0}': valid modes are 'insert', 'normal', 'prompt'")]
    InvalidMode(String),

    /// Binding conflicts with an existing plugin binding
    #[error("binding conflict: key sequence '{sequence}' is already bound to '{existing_command}' in plugin '{plugin}'")]
    Conflict {
        sequence: String,
        existing_command: String,
        plugin: String,
    },

    /// Builder validation failed (missing required fields)
    #[error("binding builder validation failed: {0}")]
    BuilderValidation(String),
}

/// A plugin keybinding specification
///
/// This struct represents a complete keybinding that a plugin wants to register.
/// It includes the key sequence, command, and context (which modes it's active in).
///
/// # Note
///
/// Plugins should use [`PluginBindingBuilder`] to create bindings rather than
/// constructing this struct directly.
///
/// # Examples
///
/// ```
/// use termide_plugin_api::input::{PluginBinding, BindingContext, EditorMode};
///
/// let binding = PluginBinding {
///     sequence: "Ctrl+S".to_string(),
///     command: "my-plugin.save".to_string(),
///     context: BindingContext::Global,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginBinding {
    /// The key sequence string (e.g., "Ctrl+S", "g d")
    pub sequence: String,
    /// The command string (e.g., "format", "goto_definition")
    /// Will be automatically namespaced with plugin name if not already namespaced
    pub command: String,
    /// The context specifying when this binding is active
    pub context: BindingContext,
}

/// Builder for creating plugin keybindings with a fluent API
///
/// Provides an ergonomic, type-safe way to construct keybindings. The builder
/// ensures all required fields are set and provides convenient methods for common patterns.
///
/// # Required Fields
///
/// - Plugin name (set via `new()`)
/// - Key sequence (set via `bind()`)
/// - Command (set via `bind()`)
///
/// # Examples
///
/// ```
/// use termide_plugin_api::input::PluginBindingBuilder;
///
/// // Global binding
/// let binding = PluginBindingBuilder::new("my-plugin")
///     .bind("Ctrl+S", "save")
///     .global()
///     .build()
///     .unwrap();
///
/// // Mode-specific binding
/// let binding = PluginBindingBuilder::new("my-plugin")
///     .bind("g d", "goto_definition")
///     .in_mode("normal")
///     .build()
///     .unwrap();
///
/// // Multi-mode binding
/// let binding = PluginBindingBuilder::new("my-plugin")
///     .bind("Ctrl+/", "toggle_comment")
///     .in_modes(&["insert", "normal"])
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct PluginBindingBuilder {
    plugin_name: String,
    sequence: Option<String>,
    command: Option<String>,
    context: BindingContext,
}

impl PluginBindingBuilder {
    /// Creates a new builder for the specified plugin
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - The name of your plugin (used for command namespacing)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide_plugin_api::input::PluginBindingBuilder;
    ///
    /// let builder = PluginBindingBuilder::new("my-plugin");
    /// ```
    pub fn new(plugin_name: impl Into<String>) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            sequence: None,
            command: None,
            context: BindingContext::Global, // Default to global
        }
    }

    /// Sets the key sequence and command for this binding
    ///
    /// # Arguments
    ///
    /// * `sequence` - The key sequence string (e.g., "Ctrl+S", "g d")
    /// * `command` - The command name (will be auto-namespaced with plugin name)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide_plugin_api::input::PluginBindingBuilder;
    ///
    /// let builder = PluginBindingBuilder::new("my-plugin")
    ///     .bind("Ctrl+K", "show_info");
    /// ```
    pub fn bind(mut self, sequence: impl Into<String>, command: impl Into<String>) -> Self {
        self.sequence = Some(sequence.into());
        self.command = Some(command.into());
        self
    }

    /// Sets the binding to be active in a single mode
    ///
    /// # Arguments
    ///
    /// * `mode` - Mode name: "insert", "normal", or "prompt" (case-insensitive)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide_plugin_api::input::PluginBindingBuilder;
    ///
    /// let binding = PluginBindingBuilder::new("my-plugin")
    ///     .bind("g d", "goto_definition")
    ///     .in_mode("normal")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn in_mode(mut self, mode: impl AsRef<str>) -> Self {
        // Validation happens in build() to avoid returning Result here
        // Store as string for now, will parse in build()
        let mode_str = mode.as_ref().to_string();
        self.context = BindingContext::Mode(
            EditorMode::from_str(&mode_str).unwrap_or(EditorMode::Normal)
        );
        self
    }

    /// Sets the binding to be active in multiple modes
    ///
    /// # Arguments
    ///
    /// * `modes` - Slice of mode names: "insert", "normal", "prompt" (case-insensitive)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide_plugin_api::input::PluginBindingBuilder;
    ///
    /// let binding = PluginBindingBuilder::new("my-plugin")
    ///     .bind("Ctrl+/", "toggle_comment")
    ///     .in_modes(&["insert", "normal"])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn in_modes(mut self, modes: &[impl AsRef<str>]) -> Self {
        let parsed_modes: Vec<EditorMode> = modes
            .iter()
            .filter_map(|m| EditorMode::from_str(m.as_ref()).ok())
            .collect();

        self.context = if parsed_modes.is_empty() {
            BindingContext::Global
        } else {
            BindingContext::Modes(parsed_modes)
        };
        self
    }

    /// Sets the binding to be global (active in Insert and Normal modes, not Prompt)
    ///
    /// This is the default context if no mode methods are called.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide_plugin_api::input::PluginBindingBuilder;
    ///
    /// let binding = PluginBindingBuilder::new("my-plugin")
    ///     .bind("Ctrl+S", "save")
    ///     .global()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn global(mut self) -> Self {
        self.context = BindingContext::Global;
        self
    }

    /// Builds the final `PluginBinding`, validating all required fields
    ///
    /// # Errors
    ///
    /// Returns [`BindingError::BuilderValidation`] if:
    /// - Sequence is not set (call `.bind()` first)
    /// - Command is not set (call `.bind()` first)
    /// - Sequence is empty
    /// - Command is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use termide_plugin_api::input::{PluginBindingBuilder, BindingError};
    ///
    /// // Valid binding
    /// let binding = PluginBindingBuilder::new("my-plugin")
    ///     .bind("Ctrl+K", "show_info")
    ///     .global()
    ///     .build()
    ///     .unwrap();
    ///
    /// // Missing sequence/command
    /// let result = PluginBindingBuilder::new("my-plugin")
    ///     .global()
    ///     .build();
    /// assert!(result.is_err());
    /// ```
    pub fn build(self) -> Result<PluginBinding, BindingError> {
        // Validate required fields
        let sequence = self.sequence.ok_or_else(|| {
            BindingError::BuilderValidation(
                "sequence is required - call .bind(sequence, command)".to_string()
            )
        })?;

        let command = self.command.ok_or_else(|| {
            BindingError::BuilderValidation(
                "command is required - call .bind(sequence, command)".to_string()
            )
        })?;

        // Validate non-empty
        if sequence.trim().is_empty() {
            return Err(BindingError::EmptySequence);
        }

        if command.trim().is_empty() {
            return Err(BindingError::EmptyCommand);
        }

        // Auto-namespace command if not already namespaced
        let namespaced_command = if command.contains('.') {
            command
        } else {
            format!("{}.{}", self.plugin_name, command)
        };

        Ok(PluginBinding {
            sequence,
            command: namespaced_command,
            context: self.context,
        })
    }
}

/// Trait for registering plugin keybindings
///
/// This trait defines the interface that plugins use to register their keybindings.
/// The editor implements this trait for its internal registry, but plugins only see
/// this abstract interface.
///
/// # Implementation Note
///
/// Plugins should not implement this trait. It is implemented by the editor's
/// internal keybinding registry.
///
/// # Examples
///
/// ```rust,no_run
/// use termide_plugin_api::input::{PluginInputExtension, PluginBindingBuilder};
///
/// fn register_bindings(registry: &mut impl PluginInputExtension) -> Result<(), Box<dyn std::error::Error>> {
///     registry.register_keybinding(
///         PluginBindingBuilder::new("my-plugin")
///             .bind("Ctrl+K", "show_info")
///             .global()
///             .build()?
///     )?;
///     Ok(())
/// }
/// ```
pub trait PluginInputExtension {
    /// Registers a plugin keybinding
    ///
    /// The binding will be registered with `Plugin` priority (10), which means:
    /// - It overrides default editor bindings (priority 0)
    /// - It is overridden by user customizations (priority 20)
    ///
    /// # Arguments
    ///
    /// * `binding` - The plugin binding to register
    ///
    /// # Errors
    ///
    /// Returns [`BindingError`] if:
    /// - The binding conflicts with an existing plugin binding (same sequence in same context)
    /// - The key sequence is invalid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use termide_plugin_api::input::{PluginInputExtension, PluginBindingBuilder};
    ///
    /// fn setup(registry: &mut impl PluginInputExtension) -> Result<(), Box<dyn std::error::Error>> {
    ///     let binding = PluginBindingBuilder::new("my-plugin")
    ///         .bind("Ctrl+K", "show_info")
    ///         .global()
    ///         .build()?;
    ///
    ///     registry.register_keybinding(binding)?;
    ///     Ok(())
    /// }
    /// ```
    fn register_keybinding(&mut self, binding: PluginBinding) -> Result<(), BindingError>;
}
