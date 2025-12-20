//! Unit tests for InternalClipboard struct

use crate::clipboard::{ClipboardError, ClipboardProvider, InternalClipboard};

#[test]
fn test_new_creates_empty_clipboard() {
    let mut clipboard = InternalClipboard::new();

    // Empty clipboard should return NotAvailable error
    let result = clipboard.get_text();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ClipboardError::NotAvailable);
}

#[test]
fn test_default_creates_empty_clipboard() {
    let mut clipboard = InternalClipboard::default();

    // Default clipboard should also be empty
    let result = clipboard.get_text();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ClipboardError::NotAvailable);
}

#[test]
fn test_set_text_and_get_text_cycle() {
    let mut clipboard = InternalClipboard::new();
    let test_text = "Hello, clipboard!";

    // Set text
    let set_result = clipboard.set_text(test_text);
    assert!(set_result.is_ok());

    // Get text should return the same content
    let get_result = clipboard.get_text();
    assert!(get_result.is_ok());
    assert_eq!(get_result.unwrap(), test_text);
}

#[test]
fn test_set_text_always_succeeds() {
    let mut clipboard = InternalClipboard::new();

    // Set text should always succeed
    assert!(clipboard.set_text("First").is_ok());
    assert!(clipboard.set_text("Second").is_ok());
    assert!(clipboard.set_text("").is_ok());
    assert!(clipboard.set_text("A very long string with lots of content...").is_ok());
}

#[test]
fn test_overwrite_behavior() {
    let mut clipboard = InternalClipboard::new();

    // Set first text
    clipboard.set_text("First").unwrap();
    assert_eq!(clipboard.get_text().unwrap(), "First");

    // Set second text - should replace first
    clipboard.set_text("Second").unwrap();
    assert_eq!(clipboard.get_text().unwrap(), "Second");

    // Set third text - should replace second
    clipboard.set_text("Third").unwrap();
    assert_eq!(clipboard.get_text().unwrap(), "Third");
}

#[test]
fn test_empty_string_handling() {
    let mut clipboard = InternalClipboard::new();

    // Empty string is valid content (different from None)
    clipboard.set_text("").unwrap();

    // Should succeed and return empty string
    let result = clipboard.get_text();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[test]
fn test_multiline_text() {
    let mut clipboard = InternalClipboard::new();
    let multiline = "Line 1\nLine 2\nLine 3";

    clipboard.set_text(multiline).unwrap();
    assert_eq!(clipboard.get_text().unwrap(), multiline);
}

#[test]
fn test_unicode_text() {
    let mut clipboard = InternalClipboard::new();
    let unicode = "Hello 世界 🌍 مرحبا";

    clipboard.set_text(unicode).unwrap();
    assert_eq!(clipboard.get_text().unwrap(), unicode);
}

#[test]
fn test_large_text() {
    let mut clipboard = InternalClipboard::new();
    // Create a large string (10KB)
    let large_text = "A".repeat(10_000);

    clipboard.set_text(&large_text).unwrap();
    assert_eq!(clipboard.get_text().unwrap(), large_text);
}

#[test]
fn test_special_characters() {
    let mut clipboard = InternalClipboard::new();
    let special = "Tab:\t Newline:\n Carriage Return:\r Null:\0";

    clipboard.set_text(special).unwrap();
    assert_eq!(clipboard.get_text().unwrap(), special);
}

#[test]
fn test_clipboard_is_send() {
    // This test verifies that InternalClipboard implements Send
    // which is required by the ClipboardProvider trait
    fn assert_send<T: Send>() {}
    assert_send::<InternalClipboard>();
}

#[test]
fn test_multiple_get_calls() {
    let mut clipboard = InternalClipboard::new();
    clipboard.set_text("test").unwrap();

    // Multiple get calls should return the same value
    assert_eq!(clipboard.get_text().unwrap(), "test");
    assert_eq!(clipboard.get_text().unwrap(), "test");
    assert_eq!(clipboard.get_text().unwrap(), "test");
}

#[test]
fn test_get_text_error_on_empty() {
    let mut clipboard = InternalClipboard::new();

    // Verify specific error type
    match clipboard.get_text() {
        Err(ClipboardError::NotAvailable) => {
            // Expected error
        }
        Ok(_) => panic!("Expected NotAvailable error, got Ok"),
        Err(e) => panic!("Expected NotAvailable error, got {:?}", e),
    }
}

#[test]
fn test_set_after_get_on_empty() {
    let mut clipboard = InternalClipboard::new();

    // Try to get from empty clipboard
    assert!(clipboard.get_text().is_err());

    // Set text should still work
    clipboard.set_text("Now it has content").unwrap();
    assert_eq!(clipboard.get_text().unwrap(), "Now it has content");
}
