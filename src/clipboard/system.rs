//! System clipboard implementation using copypasta
//!
//! This module provides integration with the system clipboard across different platforms
//! (Linux/X11, macOS, Windows) using the copypasta crate. It implements the `ClipboardProvider`
//! trait to provide consistent clipboard access regardless of the underlying platform.
//!
//! # Platform Support
//!
//! - **Linux**: Requires X11 libraries (libxcb). May fail in headless environments.
//! - **macOS**: Uses Cocoa frameworks. Generally works in all environments.
//! - **Windows**: Uses Windows API. Generally works in all environments.
//!
//! # Initialization
//!
//! System clipboard initialization can fail if:
//! - No display server is available (Linux without X11/Wayland)
//! - No clipboard daemon is running
//! - Application doesn't have clipboard permissions
//!
//! Use `SystemClipboard::new()` which returns a `Result` to handle initialization failures.
//!
//! # Example
//!
//! ```rust,no_run
//! use termide::clipboard::{ClipboardProvider, ClipboardError};
//! use termide::clipboard::SystemClipboard;
//!
//! // Attempt to create system clipboard
//! match SystemClipboard::new() {
//!     Ok(mut clipboard) => {
//!         // Set text
//!         clipboard.set_text("Hello from TermIDE!").unwrap();
//!
//!         // Get text
//!         let text = clipboard.get_text().unwrap();
//!         println!("Clipboard: {}", text);
//!     }
//!     Err(e) => {
//!         eprintln!("System clipboard not available: {}", e);
//!         // Fall back to InternalClipboard
//!     }
//! }
//! ```

use super::{ClipboardError, ClipboardProvider};
use copypasta::{ClipboardContext, ClipboardProvider as CopypastaProvider};

/// System clipboard implementation using copypasta
///
/// `SystemClipboard` wraps the copypasta `ClipboardContext` to provide platform-specific
/// system clipboard access. It implements the `ClipboardProvider` trait for consistent
/// clipboard operations across the application.
///
/// # Platform Behavior
///
/// - **Linux**: Accesses the standard clipboard selection (not primary selection).
///   Requires X11 libraries at build time and X11/Wayland at runtime.
/// - **macOS**: Uses the system pasteboard.
/// - **Windows**: Uses the Windows clipboard API.
///
/// # Thread Safety
///
/// While `SystemClipboard` implements `Send`, most platform clipboard implementations
/// are not thread-safe. Each thread should use its own instance or external synchronization.
///
/// # Initialization Failures
///
/// The `new()` constructor may fail with:
/// - `ClipboardError::NotAvailable`: No clipboard daemon or display server
/// - `ClipboardError::SystemError`: Other platform-specific initialization issues
///
/// # Example
///
/// ```rust,no_run
/// use termide::clipboard::{ClipboardProvider, SystemClipboard};
///
/// let mut clipboard = SystemClipboard::new()
///     .expect("Failed to initialize system clipboard");
///
/// clipboard.set_text("Copy this!").unwrap();
/// let text = clipboard.get_text().unwrap();
/// assert_eq!(text, "Copy this!");
/// ```
pub struct SystemClipboard {
    /// Underlying copypasta clipboard context
    context: ClipboardContext,
}

impl SystemClipboard {
    /// Creates a new system clipboard instance
    ///
    /// This method attempts to initialize the platform-specific clipboard implementation.
    /// It may fail if the clipboard is not available on the current system.
    ///
    /// # Returns
    ///
    /// - `Ok(SystemClipboard)` if clipboard initialization succeeds
    /// - `Err(ClipboardError::NotAvailable)` if no clipboard daemon or display server exists
    /// - `Err(ClipboardError::SystemError)` for other initialization failures
    ///
    /// # Platform-Specific Behavior
    ///
    /// - **Linux**: Requires X11 display server or Wayland with clipboard support
    /// - **macOS**: Generally always succeeds unless running in severely restricted environment
    /// - **Windows**: Generally always succeeds
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use termide::clipboard::{ClipboardError, SystemClipboard};
    ///
    /// match SystemClipboard::new() {
    ///     Ok(clipboard) => println!("System clipboard initialized"),
    ///     Err(ClipboardError::NotAvailable) => {
    ///         eprintln!("Clipboard not available - using fallback");
    ///     }
    ///     Err(e) => eprintln!("Clipboard error: {}", e),
    /// }
    /// ```
    pub fn new() -> Result<Self, ClipboardError> {
        // Attempt to create clipboard context
        // This can fail on Linux without X11, headless environments, etc.
        ClipboardContext::new()
            .map(|context| SystemClipboard { context })
            .map_err(|e| {
                // Map copypasta error to our ClipboardError
                // Most initialization failures are due to unavailable clipboard
                let error_msg = e.to_string();
                if error_msg.contains("not available")
                    || error_msg.contains("no display")
                    || error_msg.contains("DISPLAY")
                {
                    ClipboardError::NotAvailable
                } else {
                    ClipboardError::SystemError(error_msg)
                }
            })
    }
}

impl ClipboardProvider for SystemClipboard {
    /// Retrieves text from the system clipboard
    ///
    /// # Returns
    ///
    /// - `Ok(String)` containing the clipboard text if available and valid UTF-8
    /// - `Err(ClipboardError::NotAvailable)` if clipboard is empty or not accessible
    /// - `Err(ClipboardError::InvalidData)` if clipboard contains non-UTF-8 data
    /// - `Err(ClipboardError::AccessDenied)` if permission is denied
    /// - `Err(ClipboardError::SystemError)` for other platform-specific errors
    ///
    /// # Platform-Specific Behavior
    ///
    /// - **Linux**: Reads from the CLIPBOARD selection (not PRIMARY)
    /// - **macOS**: Reads from the system pasteboard
    /// - **Windows**: Reads from the Windows clipboard
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use termide::clipboard::{ClipboardProvider, ClipboardError, SystemClipboard};
    ///
    /// let mut clipboard = SystemClipboard::new().unwrap();
    ///
    /// match clipboard.get_text() {
    ///     Ok(text) => println!("Clipboard contains: {}", text),
    ///     Err(ClipboardError::NotAvailable) => println!("Clipboard is empty"),
    ///     Err(ClipboardError::InvalidData) => println!("Clipboard contains non-text data"),
    ///     Err(e) => eprintln!("Error reading clipboard: {}", e),
    /// }
    /// ```
    fn get_text(&mut self) -> Result<String, ClipboardError> {
        self.context.get_contents().map_err(|e| {
            let error_msg = e.to_string().to_lowercase();

            // Map copypasta errors to our ClipboardError variants
            if error_msg.contains("not available") || error_msg.contains("empty") {
                ClipboardError::NotAvailable
            } else if error_msg.contains("denied") || error_msg.contains("permission") {
                ClipboardError::AccessDenied
            } else if error_msg.contains("invalid") || error_msg.contains("utf") {
                ClipboardError::InvalidData
            } else {
                ClipboardError::SystemError(e.to_string())
            }
        })
    }

    /// Writes text to the system clipboard
    ///
    /// This method replaces any existing clipboard content with the provided text.
    /// The text will be available to other applications on the system.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to copy to the clipboard
    ///
    /// # Returns
    ///
    /// - `Ok(())` if text was successfully copied to clipboard
    /// - `Err(ClipboardError::NotAvailable)` if clipboard is not accessible
    /// - `Err(ClipboardError::AccessDenied)` if permission is denied
    /// - `Err(ClipboardError::SystemError)` for other platform-specific errors
    ///
    /// # Platform-Specific Behavior
    ///
    /// - **Linux**: Writes to the CLIPBOARD selection. The data remains available
    ///   even after the application exits (clipboard manager dependent).
    /// - **macOS**: Writes to the system pasteboard. Data persists after exit.
    /// - **Windows**: Writes to the Windows clipboard. Data persists after exit.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use termide::clipboard::{ClipboardProvider, SystemClipboard};
    ///
    /// let mut clipboard = SystemClipboard::new().unwrap();
    ///
    /// match clipboard.set_text("Important data") {
    ///     Ok(()) => println!("Text copied to clipboard"),
    ///     Err(e) => eprintln!("Failed to copy: {}", e),
    /// }
    /// ```
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.context.set_contents(text.to_owned()).map_err(|e| {
            let error_msg = e.to_string().to_lowercase();

            // Map copypasta errors to our ClipboardError variants
            if error_msg.contains("not available") {
                ClipboardError::NotAvailable
            } else if error_msg.contains("denied") || error_msg.contains("permission") {
                ClipboardError::AccessDenied
            } else {
                ClipboardError::SystemError(e.to_string())
            }
        })
    }
}
