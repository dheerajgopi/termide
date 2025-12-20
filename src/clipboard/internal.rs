//! Internal clipboard implementation for fallback scenarios
//!
//! This module provides a simple in-memory clipboard that stores a single string value.
//! It serves as a fallback when the system clipboard is unavailable, such as in:
//! - Headless environments
//! - CI/CD pipelines
//! - Systems without clipboard daemon running
//! - SSH sessions without X11 forwarding
//!
//! # Example
//!
//! ```rust
//! use termide::clipboard::{ClipboardProvider, ClipboardError};
//! use termide::clipboard::InternalClipboard;
//!
//! let mut clipboard = InternalClipboard::default();
//!
//! // Set text - always succeeds
//! clipboard.set_text("Hello, world!").unwrap();
//!
//! // Get text - returns the stored content
//! let text = clipboard.get_text().unwrap();
//! assert_eq!(text, "Hello, world!");
//! ```

use super::{ClipboardError, ClipboardProvider};

/// Simple in-memory clipboard implementation
///
/// `InternalClipboard` provides a fallback clipboard that stores content in memory.
/// It never fails for `set_text` operations and is thread-safe through Rust's
/// ownership system.
///
/// # Thread Safety
///
/// While `InternalClipboard` implements `Send`, each instance is designed for
/// single-threaded access. Multiple threads should use separate instances or
/// external synchronization.
///
/// # Memory Management
///
/// Content is automatically freed when replaced (Rust RAII). No manual cleanup needed.
///
/// # Example
///
/// ```rust
/// use termide::clipboard::{ClipboardProvider, InternalClipboard};
///
/// let mut clipboard = InternalClipboard::default();
///
/// // First set
/// clipboard.set_text("First").unwrap();
/// assert_eq!(clipboard.get_text().unwrap(), "First");
///
/// // Overwrite - previous content is automatically freed
/// clipboard.set_text("Second").unwrap();
/// assert_eq!(clipboard.get_text().unwrap(), "Second");
/// ```
#[derive(Debug, Default)]
pub struct InternalClipboard {
    /// Stored clipboard content
    ///
    /// `None` represents an empty clipboard state.
    content: Option<String>,
}

impl InternalClipboard {
    /// Creates a new empty internal clipboard
    ///
    /// # Example
    ///
    /// ```rust
    /// use termide::clipboard::InternalClipboard;
    ///
    /// let clipboard = InternalClipboard::new();
    /// ```
    pub fn new() -> Self {
        Self { content: None }
    }
}

impl ClipboardProvider for InternalClipboard {
    /// Retrieves text from the internal clipboard
    ///
    /// # Returns
    ///
    /// - `Ok(String)` containing the clipboard text if content exists
    /// - `Err(ClipboardError::NotAvailable)` if clipboard is empty
    ///
    /// # Example
    ///
    /// ```rust
    /// use termide::clipboard::{ClipboardProvider, ClipboardError, InternalClipboard};
    ///
    /// let mut clipboard = InternalClipboard::new();
    ///
    /// // Empty clipboard returns NotAvailable
    /// match clipboard.get_text() {
    ///     Err(ClipboardError::NotAvailable) => println!("Clipboard is empty"),
    ///     _ => panic!("Expected NotAvailable error"),
    /// }
    ///
    /// // After setting text, get_text succeeds
    /// clipboard.set_text("test").unwrap();
    /// assert_eq!(clipboard.get_text().unwrap(), "test");
    /// ```
    fn get_text(&mut self) -> Result<String, ClipboardError> {
        self.content
            .clone()
            .ok_or(ClipboardError::NotAvailable)
    }

    /// Writes text to the internal clipboard
    ///
    /// This operation always succeeds and replaces any previous content.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to store in the clipboard
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`. This implementation never fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use termide::clipboard::{ClipboardProvider, InternalClipboard};
    ///
    /// let mut clipboard = InternalClipboard::new();
    ///
    /// // First set
    /// clipboard.set_text("Hello").unwrap();
    /// assert_eq!(clipboard.get_text().unwrap(), "Hello");
    ///
    /// // Overwrite previous content
    /// clipboard.set_text("World").unwrap();
    /// assert_eq!(clipboard.get_text().unwrap(), "World");
    /// ```
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.content = Some(text.to_string());
        Ok(())
    }
}
