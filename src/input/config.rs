//! User keybinding configuration loader
//!
//! This module provides functionality to load user-customized keybindings from TOML
//! configuration files. User bindings are registered with `Priority::User`, allowing
//! them to override plugin and default bindings.
//!
//! # Configuration Format
//!
//! User keybindings are defined in TOML format:
//!
//! ```toml
//! [[keybindings]]
//! sequence = "Ctrl+S"
//! command = "file.save"
//! # mode is optional - if omitted, binding is global
//!
//! [[keybindings]]
//! sequence = "d d"
//! command = "delete_char"
//! mode = "normal"  # mode-specific binding
//! ```
//!
//! # Features
//!
//! - **Validation**: Detailed error messages for malformed sequences or commands
//! - **Graceful Error Handling**: Parse errors don't crash the application
//! - **Priority System**: User bindings automatically get `Priority::User`
//!
//! # Examples
//!
//! ```no_run
//! use termide::input::config::load_user_keybindings;
//! use termide::input::registry::KeyBindingRegistry;
//! use std::path::Path;
//! use std::time::Duration;
//!
//! let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
//! let config_path = Path::new("~/.config/termide/config.toml");
//!
//! match load_user_keybindings(&mut registry, config_path) {
//!     Ok(count) => println!("Loaded {} user keybindings", count),
//!     Err(e) => eprintln!("Failed to load config: {}", e),
//! }
//! ```

use crate::editor::EditorMode;
use crate::input::keybinding::{BindingContext, KeyBinding, KeySequence, Priority};
use crate::input::registry::{BindingError, KeyBindingRegistry};
use crate::input::{CommandParseError, EditorCommand, ParseError};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

/// Error type for configuration loading
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to read the configuration file
    #[error("failed to read config file '{path}': {source}")]
    ReadError {
        path: String,
        source: std::io::Error,
    },

    /// Failed to parse the TOML configuration
    #[error("failed to parse TOML in '{path}': {source}")]
    TomlParseError { path: String, source: toml::de::Error },

    /// Invalid key sequence in configuration
    #[error("invalid key sequence '{sequence}' in keybinding #{index}: {reason}")]
    InvalidSequence {
        index: usize,
        sequence: String,
        reason: String,
    },

    /// Invalid command in configuration
    #[error("invalid command '{command}' in keybinding #{index}: {reason}")]
    InvalidCommand {
        index: usize,
        command: String,
        reason: String,
    },

    /// Invalid mode in configuration
    #[error("invalid mode '{mode}' in keybinding #{index}: valid modes are 'insert', 'normal', 'prompt'")]
    InvalidMode {
        index: usize,
        mode: String,
    },

    /// Binding conflict when registering
    #[error("binding conflict in keybinding #{index}: {source}")]
    BindingConflict {
        index: usize,
        source: BindingError,
    },
}

/// Root configuration structure
///
/// This struct represents the complete TOML configuration file structure.
/// Currently, it only contains keybindings, but can be extended in the future
/// to include other configuration options (theme, window size, etc.).
///
/// # Examples
///
/// ```toml
/// [[keybindings]]
/// sequence = "Ctrl+S"
/// command = "file.save"
///
/// [[keybindings]]
/// sequence = "d d"
/// command = "delete_char"
/// mode = "normal"
/// ```
#[derive(Debug, Deserialize)]
pub struct KeybindingConfig {
    /// List of user-defined keybindings
    #[serde(default)]
    pub keybindings: Vec<UserBinding>,
}

/// User-defined keybinding from configuration file
///
/// Represents a single keybinding entry in the TOML configuration.
/// All fields are strings that will be validated and converted to
/// strongly-typed internal representations.
///
/// # Fields
///
/// - `sequence`: Key sequence string (e.g., "Ctrl+S", "d d")
/// - `command`: Command name (e.g., "file.save", "delete_char")
/// - `mode`: Optional mode restriction (e.g., "insert", "normal", "prompt")
///
/// # Examples
///
/// ```toml
/// # Global binding (no mode specified)
/// [[keybindings]]
/// sequence = "Ctrl+S"
/// command = "file.save"
///
/// # Mode-specific binding
/// [[keybindings]]
/// sequence = "i"
/// command = "mode.insert"
/// mode = "normal"
///
/// # Multi-key sequence
/// [[keybindings]]
/// sequence = "d d"
/// command = "delete_char"
/// mode = "normal"
/// ```
#[derive(Debug, Deserialize)]
pub struct UserBinding {
    /// Key sequence string (e.g., "Ctrl+S", "d d")
    pub sequence: String,
    /// Command name (e.g., "file.save", "delete_char")
    pub command: String,
    /// Optional mode restriction (e.g., "insert", "normal", "prompt")
    #[serde(default)]
    pub mode: Option<String>,
}

/// Load user keybindings from a TOML configuration file
///
/// This function reads the configuration file, parses it, validates each binding,
/// and registers valid bindings with `Priority::User`. Invalid bindings generate
/// warnings but don't prevent other bindings from loading.
///
/// # Arguments
///
/// * `registry` - The keybinding registry to register bindings into
/// * `path` - Path to the TOML configuration file
///
/// # Returns
///
/// - `Ok(usize)` - Number of successfully loaded bindings
/// - `Err(ConfigError)` - Fatal error (file not readable, invalid TOML, etc.)
///
/// # Error Handling
///
/// This function uses a "fail-fast for fatal errors, continue for entry errors" approach:
///
/// - **Fatal errors** (file not found, invalid TOML): Return `Err` immediately
/// - **Entry errors** (invalid sequence, unknown command): Log warning, skip entry, continue
///
/// This ensures that one bad entry doesn't prevent other valid entries from loading.
///
/// # Examples
///
/// ```no_run
/// use termide::input::config::load_user_keybindings;
/// use termide::input::registry::KeyBindingRegistry;
/// use std::path::Path;
/// use std::time::Duration;
///
/// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
/// let config_path = Path::new("~/.config/termide/config.toml");
///
/// match load_user_keybindings(&mut registry, config_path) {
///     Ok(count) => println!("Loaded {} user keybindings", count),
///     Err(e) => {
///         eprintln!("Failed to load config: {}", e);
///         // Continue with defaults
///     }
/// }
/// ```
///
/// # Validation
///
/// Each binding is validated for:
/// - Valid key sequence syntax (uses `KeySequence::from_str`)
/// - Valid command name (uses `EditorCommand::from_str`)
/// - Valid mode name if specified (case-insensitive)
/// - No conflicts at `Priority::User` level
pub fn load_user_keybindings(
    registry: &mut KeyBindingRegistry,
    path: &Path,
) -> Result<usize, ConfigError> {
    // Read the file
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::ReadError {
        path: path.display().to_string(),
        source,
    })?;

    // Parse TOML
    let config: KeybindingConfig =
        toml::from_str(&contents).map_err(|source| ConfigError::TomlParseError {
            path: path.display().to_string(),
            source,
        })?;

    // Load each binding, collecting successes and logging failures
    let mut loaded_count = 0;

    for (index, user_binding) in config.keybindings.iter().enumerate() {
        let binding_num = index + 1; // 1-indexed for user-friendly messages

        match load_single_binding(user_binding, binding_num) {
            Ok(binding) => {
                // Try to register the binding
                match registry.register(binding) {
                    Ok(()) => {
                        loaded_count += 1;
                    }
                    Err(e) => {
                        // Log warning but continue with other bindings
                        eprintln!(
                            "Warning: {}",
                            ConfigError::BindingConflict {
                                index: binding_num,
                                source: e
                            }
                        );
                    }
                }
            }
            Err(e) => {
                // Log warning but continue with other bindings
                eprintln!("Warning: {}", e);
            }
        }
    }

    Ok(loaded_count)
}

/// Load and validate a single user binding
///
/// Converts a `UserBinding` (string-based) to a `KeyBinding` (strongly-typed).
/// Performs all necessary validation and type conversion.
///
/// # Arguments
///
/// * `user_binding` - The user binding from TOML config
/// * `index` - Binding number (1-indexed) for error messages
///
/// # Returns
///
/// - `Ok(KeyBinding)` - Successfully validated binding
/// - `Err(ConfigError)` - Validation error with detailed message
fn load_single_binding(
    user_binding: &UserBinding,
    index: usize,
) -> Result<KeyBinding, ConfigError> {
    // Parse key sequence
    let sequence = KeySequence::from_str(&user_binding.sequence).map_err(|e| {
        ConfigError::InvalidSequence {
            index,
            sequence: user_binding.sequence.clone(),
            reason: format_parse_error(&e),
        }
    })?;

    // Parse command
    let command = EditorCommand::from_str(&user_binding.command).map_err(|e| {
        ConfigError::InvalidCommand {
            index,
            command: user_binding.command.clone(),
            reason: format_command_error(&e),
        }
    })?;

    // Parse mode if specified
    let context = if let Some(mode_str) = &user_binding.mode {
        let mode = parse_mode(mode_str).map_err(|_| ConfigError::InvalidMode {
            index,
            mode: mode_str.clone(),
        })?;
        BindingContext::Mode(mode)
    } else {
        BindingContext::Global
    };

    // Create binding with User priority
    Ok(KeyBinding::new(sequence, command, context, Priority::User))
}

/// Parse a mode string (case-insensitive with whitespace trimming)
///
/// # Examples
///
/// ```
/// # use termide::input::config::parse_mode;
/// # use termide::editor::EditorMode;
/// assert_eq!(parse_mode("insert").unwrap(), EditorMode::Insert);
/// assert_eq!(parse_mode("NORMAL").unwrap(), EditorMode::Normal);
/// assert_eq!(parse_mode("  prompt  ").unwrap(), EditorMode::Prompt);
/// assert!(parse_mode("invalid").is_err());
/// ```
pub fn parse_mode(s: &str) -> Result<EditorMode, ()> {
    match s.trim().to_lowercase().as_str() {
        "insert" => Ok(EditorMode::Insert),
        "normal" => Ok(EditorMode::Normal),
        "prompt" => Ok(EditorMode::Prompt),
        _ => Err(()),
    }
}

/// Format a key sequence parse error into a user-friendly message
fn format_parse_error(error: &ParseError) -> String {
    match error {
        ParseError::EmptyInput => "sequence cannot be empty".to_string(),
        ParseError::EmptyPattern => "incomplete key pattern (e.g., 'Ctrl+' with no key)".to_string(),
        ParseError::UnknownModifier(m) => {
            format!("unknown modifier '{}' (valid: Ctrl, Shift, Alt, Super)", m)
        }
        ParseError::UnknownKey(k) => {
            format!("unknown key name '{}' (check spelling or use special key names like Enter, Esc, Tab)", k)
        }
        ParseError::InvalidFormat(f) => {
            format!("invalid format '{}' (expected 'Ctrl+S' or 'd d')", f)
        }
    }
}

/// Format a command parse error into a user-friendly message
fn format_command_error(error: &CommandParseError) -> String {
    match error {
        CommandParseError::EmptyCommand => "command cannot be empty".to_string(),
        CommandParseError::UnknownCommand(cmd) => format!("unknown command '{}'", cmd),
        CommandParseError::InvalidParameter { command, param, reason } => {
            format!("invalid parameter '{}' for command '{}': {}", param, command, reason)
        }
        CommandParseError::InvalidPluginCommandFormat(fmt) => {
            format!("invalid plugin command format '{}'", fmt)
        }
    }
}

/// Get the platform-specific path to the user's keybinding configuration file
///
/// This function returns the standard configuration file path for the current platform:
/// - Linux/macOS: `~/.config/termide/config.toml`
/// - Windows: `%APPDATA%\termide\config.toml`
///
/// # Returns
///
/// - `Some(PathBuf)` - Path to the config file if a config directory could be determined
/// - `None` - If the system config directory could not be determined (rare)
///
/// # Examples
///
/// ```no_run
/// use termide::input::config::get_config_path;
///
/// if let Some(path) = get_config_path() {
///     println!("Config file location: {}", path.display());
/// }
/// ```
///
/// # Platform Behavior
///
/// - **Linux**: Uses `$XDG_CONFIG_HOME/termide/config.toml` or `~/.config/termide/config.toml`
/// - **macOS**: Uses `~/Library/Application Support/termide/config.toml`
/// - **Windows**: Uses `%APPDATA%\termide\config.toml`
pub fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push("termide");
        path.push("config.toml");
        path
    })
}
