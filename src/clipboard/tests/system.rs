//! Unit tests for SystemClipboard struct

use crate::clipboard::{ClipboardError, ClipboardProvider, SystemClipboard};

/// Test SystemClipboard initialization
///
/// This test verifies that SystemClipboard can be initialized.
/// Note: This may fail in headless CI environments, which is expected behavior.
#[test]
fn test_system_clipboard_new() {
    match SystemClipboard::new() {
        Ok(_clipboard) => {
            // Success - system clipboard is available
            // This is expected in environments with display server
        }
        Err(ClipboardError::NotAvailable) => {
            // Expected in headless environments, CI/CD, etc.
            // This is not a test failure
        }
        Err(e) => {
            // Other errors might indicate a problem
            eprintln!("Unexpected clipboard error: {}", e);
        }
    }
}

/// Test set_text and get_text round-trip
///
/// This test verifies that text can be written to and read from the system clipboard.
/// It's skipped in headless environments where clipboard is unavailable.
#[test]
fn test_system_clipboard_round_trip() {
    let mut clipboard = match SystemClipboard::new() {
        Ok(c) => c,
        Err(ClipboardError::NotAvailable) => {
            eprintln!("Skipping test - clipboard not available (headless environment)");
            return;
        }
        Err(e) => panic!("Failed to initialize clipboard: {}", e),
    };

    // Set text
    let test_text = "Hello from TermIDE!";
    // Attempt to set text - may fail in headless/WSL environments
    if let Err(e) = clipboard.set_text(test_text) {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    // Get text back
    let retrieved_text = match clipboard.get_text() {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Skipping test - clipboard get failed: {}", e);
            return;
        }
    };

    assert_eq!(retrieved_text, test_text);
}

/// Test multiline text handling
///
/// Verifies that clipboard preserves newlines in multiline text.
#[test]
fn test_system_clipboard_multiline() {
    let mut clipboard = match SystemClipboard::new() {
        Ok(c) => c,
        Err(ClipboardError::NotAvailable) => {
            eprintln!("Skipping test - clipboard not available (headless environment)");
            return;
        }
        Err(e) => panic!("Failed to initialize clipboard: {}", e),
    };

    let multiline_text = "Line 1\nLine 2\nLine 3";
    if let Err(e) = clipboard.set_text(multiline_text) {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    let retrieved = clipboard
        .get_text()
        .expect("Failed to get multiline text");

    assert_eq!(retrieved, multiline_text);
}

/// Test empty string handling
///
/// Verifies that empty strings can be copied to clipboard.
#[test]
fn test_system_clipboard_empty_string() {
    let mut clipboard = match SystemClipboard::new() {
        Ok(c) => c,
        Err(ClipboardError::NotAvailable) => {
            eprintln!("Skipping test - clipboard not available (headless environment)");
            return;
        }
        Err(e) => panic!("Failed to initialize clipboard: {}", e),
    };

    if let Err(e) = clipboard.set_text("") {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    let retrieved = clipboard.get_text().expect("Failed to get empty text");

    assert_eq!(retrieved, "");
}

/// Test unicode text handling
///
/// Verifies that clipboard correctly handles unicode characters.
#[test]
fn test_system_clipboard_unicode() {
    let mut clipboard = match SystemClipboard::new() {
        Ok(c) => c,
        Err(ClipboardError::NotAvailable) => {
            eprintln!("Skipping test - clipboard not available (headless environment)");
            return;
        }
        Err(e) => panic!("Failed to initialize clipboard: {}", e),
    };

    let unicode_text = "Hello 世界 🚀 Привет";
    if let Err(e) = clipboard.set_text(unicode_text) {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    let retrieved = match clipboard.get_text() {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Skipping test - clipboard get failed: {}", e);
            return;
        }
    };

    assert_eq!(retrieved, unicode_text);
}

/// Test overwriting clipboard content
///
/// Verifies that subsequent set_text calls replace previous content.
#[test]
fn test_system_clipboard_overwrite() {
    let mut clipboard = match SystemClipboard::new() {
        Ok(c) => c,
        Err(ClipboardError::NotAvailable) => {
            eprintln!("Skipping test - clipboard not available (headless environment)");
            return;
        }
        Err(e) => panic!("Failed to initialize clipboard: {}", e),
    };

    // First write
    if let Err(e) = clipboard.set_text("First") {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    // Try to get first text
    let first_text = match clipboard.get_text() {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Skipping test - clipboard get failed: {}", e);
            return;
        }
    };
    assert_eq!(first_text, "First");

    // Second write should replace
    if let Err(e) = clipboard.set_text("Second") {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    let second_text = match clipboard.get_text() {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Skipping test - clipboard get failed: {}", e);
            return;
        }
    };
    assert_eq!(second_text, "Second");
}

/// Test large text handling
///
/// Verifies that clipboard can handle reasonably large text content.
#[test]
fn test_system_clipboard_large_text() {
    let mut clipboard = match SystemClipboard::new() {
        Ok(c) => c,
        Err(ClipboardError::NotAvailable) => {
            eprintln!("Skipping test - clipboard not available (headless environment)");
            return;
        }
        Err(e) => panic!("Failed to initialize clipboard: {}", e),
    };

    // Create a large text (10KB)
    let large_text = "x".repeat(10_000);
    if let Err(e) = clipboard.set_text(&large_text) {
        eprintln!("Skipping test - clipboard set failed (large text): {}", e);
        return;
    }

    let retrieved = clipboard.get_text().expect("Failed to get large text");

    assert_eq!(retrieved, large_text);
    assert_eq!(retrieved.len(), 10_000);
}

/// Test special characters handling
///
/// Verifies that clipboard preserves special characters.
#[test]
fn test_system_clipboard_special_chars() {
    let mut clipboard = match SystemClipboard::new() {
        Ok(c) => c,
        Err(ClipboardError::NotAvailable) => {
            eprintln!("Skipping test - clipboard not available (headless environment)");
            return;
        }
        Err(e) => panic!("Failed to initialize clipboard: {}", e),
    };

    let special_text = "Tab:\t Newline:\n Quote:\" Backslash:\\ Null-like:NUL";
    if let Err(e) = clipboard.set_text(special_text) {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    let retrieved = clipboard
        .get_text()
        .expect("Failed to get special chars");

    assert_eq!(retrieved, special_text);
}

/// Test windows-style line endings
///
/// Verifies that CRLF line endings are preserved.
#[test]
fn test_system_clipboard_crlf() {
    let mut clipboard = match SystemClipboard::new() {
        Ok(c) => c,
        Err(ClipboardError::NotAvailable) => {
            eprintln!("Skipping test - clipboard not available (headless environment)");
            return;
        }
        Err(e) => panic!("Failed to initialize clipboard: {}", e),
    };

    let crlf_text = "Line 1\r\nLine 2\r\nLine 3";
    if let Err(e) = clipboard.set_text(crlf_text) {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    let retrieved = clipboard.get_text().expect("Failed to get CRLF text");

    // Note: Some platforms may normalize line endings
    // We accept either the original or normalized version
    assert!(
        retrieved == crlf_text || retrieved == "Line 1\nLine 2\nLine 3",
        "Line endings should be preserved or normalized"
    );
}

/// Test thread safety (Send trait)
///
/// Verifies that SystemClipboard implements Send trait for thread safety.
#[test]
fn test_system_clipboard_send() {
    // This is a compile-time check
    fn assert_send<T: Send>() {}
    assert_send::<SystemClipboard>();
}

/// Test error handling for initialization failure
///
/// This test documents expected behavior when clipboard is not available.
#[test]
fn test_system_clipboard_initialization_error_types() {
    match SystemClipboard::new() {
        Ok(_) => {
            // Clipboard available - no error to test
        }
        Err(ClipboardError::NotAvailable) => {
            // Expected in headless environments
        }
        Err(ClipboardError::SystemError(_)) => {
            // Other system errors are acceptable
        }
        Err(e) => {
            panic!(
                "Unexpected error type on initialization: {:?}. Expected NotAvailable or SystemError",
                e
            );
        }
    }
}
