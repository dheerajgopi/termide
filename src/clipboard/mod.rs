//! Clipboard module for cross-platform clipboard operations
//!
//! This module provides a platform-agnostic abstraction layer for clipboard operations.
//! It defines the `ClipboardProvider` trait for reading and writing text to the system
//! clipboard, along with comprehensive error handling.
//!
//! # Architecture
//!
//! The clipboard system is designed with the following components:
//! - `ClipboardProvider` trait: Platform-agnostic interface for clipboard operations
//! - `ClipboardError`: Comprehensive error type for clipboard failures
//! - Platform-specific implementations (added in later tasks)
//! - Internal fallback clipboard for headless environments
//!
//! # Example
//!
//! ```rust,no_run
//! use termide::clipboard::{ClipboardProvider, ClipboardError};
//!
//! fn copy_text(clipboard: &mut dyn ClipboardProvider, text: &str) -> Result<(), ClipboardError> {
//!     clipboard.set_text(text)?;
//!     println!("Text copied to clipboard");
//!     Ok(())
//! }
//!
//! fn paste_text(clipboard: &mut dyn ClipboardProvider) -> Result<String, ClipboardError> {
//!     clipboard.get_text()
//! }
//! ```

mod internal;

pub use internal::InternalClipboard;

use std::fmt;

/// Error type for clipboard operations
///
/// This enum represents various failure scenarios that can occur during
/// clipboard operations across different platforms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardError {
    /// Clipboard is not available on this system
    ///
    /// This can occur in headless environments, when no clipboard daemon is running,
    /// or when the display server is not accessible.
    ///
    /// # Example Scenarios
    /// - Linux without X11/Wayland display server
    /// - SSH session without X11 forwarding
    /// - CI/CD environments
    NotAvailable,

    /// Access to the clipboard was denied
    ///
    /// This occurs when the application doesn't have permission to access
    /// the system clipboard.
    ///
    /// # Example Scenarios
    /// - Sandboxed applications without clipboard permissions
    /// - Security policies preventing clipboard access
    /// - Platform-specific permission issues
    AccessDenied,

    /// Clipboard data is invalid or corrupted
    ///
    /// This occurs when clipboard content cannot be interpreted as valid UTF-8 text
    /// or the data format is incompatible.
    ///
    /// # Example Scenarios
    /// - Non-text data in clipboard (images, binary data)
    /// - Invalid UTF-8 sequences
    /// - Corrupted clipboard data
    InvalidData,

    /// A system-level error occurred
    ///
    /// This is a catch-all for platform-specific errors that don't fit
    /// into other categories. The string contains a human-readable error message.
    ///
    /// # Example Scenarios
    /// - Operating system clipboard API failures
    /// - Internal clipboard service crashes
    /// - Unexpected platform-specific issues
    SystemError(String),
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardError::NotAvailable => {
                write!(f, "Clipboard is not available on this system")
            }
            ClipboardError::AccessDenied => {
                write!(f, "Access to clipboard was denied")
            }
            ClipboardError::InvalidData => {
                write!(f, "Clipboard data is invalid or corrupted")
            }
            ClipboardError::SystemError(msg) => {
                write!(f, "System clipboard error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ClipboardError {}

/// Platform-agnostic clipboard provider trait
///
/// This trait defines the interface for clipboard operations. Implementations
/// provide platform-specific clipboard access (system clipboard) or fallback
/// implementations (internal clipboard).
///
/// # Thread Safety
///
/// Implementors must be `Send` to support multi-threaded editor architectures.
/// However, most implementations are designed for single-threaded access per
/// clipboard instance.
///
/// # Contract
///
/// Implementations must ensure:
/// - `get_text()` returns the most recent text set via `set_text()` or from external sources
/// - `set_text()` replaces any previous clipboard content
/// - Errors are returned as `ClipboardError` variants with appropriate context
/// - Operations should be reasonably fast (< 100ms for typical use cases)
///
/// # Example Implementation
///
/// ```rust,no_run
/// use termide::clipboard::{ClipboardProvider, ClipboardError};
///
/// struct MyClipboard {
///     content: Option<String>,
/// }
///
/// impl ClipboardProvider for MyClipboard {
///     fn get_text(&mut self) -> Result<String, ClipboardError> {
///         self.content.clone().ok_or(ClipboardError::NotAvailable)
///     }
///
///     fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
///         self.content = Some(text.to_string());
///         Ok(())
///     }
/// }
/// ```
pub trait ClipboardProvider: Send {
    /// Retrieves text from the clipboard
    ///
    /// # Returns
    ///
    /// - `Ok(String)` containing the clipboard text if available
    /// - `Err(ClipboardError::NotAvailable)` if clipboard is empty or not accessible
    /// - `Err(ClipboardError::AccessDenied)` if permission is denied
    /// - `Err(ClipboardError::InvalidData)` if clipboard contains non-text data
    /// - `Err(ClipboardError::SystemError)` for other platform-specific errors
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use termide::clipboard::{ClipboardProvider, ClipboardError};
    ///
    /// fn paste(clipboard: &mut dyn ClipboardProvider) {
    ///     match clipboard.get_text() {
    ///         Ok(text) => println!("Pasted: {}", text),
    ///         Err(ClipboardError::NotAvailable) => println!("Clipboard is empty"),
    ///         Err(e) => eprintln!("Failed to paste: {}", e),
    ///     }
    /// }
    /// ```
    fn get_text(&mut self) -> Result<String, ClipboardError>;

    /// Writes text to the clipboard
    ///
    /// This method replaces any existing clipboard content with the provided text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to copy to the clipboard
    ///
    /// # Returns
    ///
    /// - `Ok(())` if text was successfully copied
    /// - `Err(ClipboardError::AccessDenied)` if permission is denied
    /// - `Err(ClipboardError::NotAvailable)` if clipboard is not accessible
    /// - `Err(ClipboardError::SystemError)` for other platform-specific errors
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use termide::clipboard::{ClipboardProvider, ClipboardError};
    ///
    /// fn copy(clipboard: &mut dyn ClipboardProvider, text: &str) {
    ///     match clipboard.set_text(text) {
    ///         Ok(()) => println!("Copied {} chars", text.len()),
    ///         Err(e) => eprintln!("Failed to copy: {}", e),
    ///     }
    /// }
    /// ```
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError>;
}

#[cfg(test)]
mod tests;
