//! # TermIDE Plugin API
//!
//! This crate provides the public API for creating TermIDE plugins. It defines
//! the interfaces and types that plugins use to extend the editor's functionality.
//!
//! ## Overview
//!
//! The Plugin API follows semantic versioning principles to ensure backward compatibility.
//! Breaking changes will be avoided when possible, and when necessary, will be clearly
//! documented with migration guides.
//!
//! ## Current Capabilities
//!
//! ### Input System Extensions (v0.1.0)
//!
//! Plugins can register custom keybindings that trigger plugin-specific commands:
//!
//! ```rust
//! use termide_plugin_api::input::{PluginInputExtension, PluginBindingBuilder};
//!
//! fn register_my_plugin_bindings(registry: &mut impl PluginInputExtension) -> Result<(), Box<dyn std::error::Error>> {
//!     // Register a global keybinding (active in Insert and Normal modes)
//!     registry.register_keybinding(
//!         PluginBindingBuilder::new("my-plugin")
//!             .bind("Ctrl+Shift+F", "format")
//!             .global()
//!             .build()?
//!     )?;
//!
//!     // Register a mode-specific keybinding (only in Normal mode)
//!     registry.register_keybinding(
//!         PluginBindingBuilder::new("my-plugin")
//!             .bind("g d", "goto_definition")
//!             .in_mode("normal")
//!             .build()?
//!     )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Plugin Command Format
//!
//! Plugin commands use a namespaced format to avoid conflicts:
//! - Format: `plugin_name.command_name`
//! - Example: `rust_analyzer.format`, `lsp.goto_definition`
//!
//! The builder automatically namespaces commands with your plugin name if not already namespaced.
//!
//! ## Priority System
//!
//! When multiple bindings match the same key sequence, priority determines which wins:
//! - **User** (priority 20): User customizations via config files
//! - **Plugin** (priority 10): Plugin-defined bindings (your bindings)
//! - **Default** (priority 0): Built-in editor bindings
//!
//! User customizations always override plugin bindings, which override defaults.
//!
//! ## Future Capabilities
//!
//! Future versions will add:
//! - UI extension points (status line, command palette)
//! - Buffer manipulation APIs
//! - LSP integration helpers
//! - Custom syntax highlighting
//!
//! ## Stability Guarantees
//!
//! - **v0.x.y**: API is experimental, breaking changes may occur with minor version bumps
//! - **v1.x.y**: Stable API with semantic versioning guarantees
//!
//! ## Examples
//!
//! See the `examples/` directory for complete plugin implementations.

pub mod input;

// Re-export commonly used types at the crate root for convenience
pub use input::{PluginInputExtension, PluginBinding, PluginBindingBuilder, BindingError};

#[cfg(test)]
mod tests;
