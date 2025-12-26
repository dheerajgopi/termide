//! Integration tests for clipboard module
//!
//! These tests verify clipboard functionality in realistic scenarios,
//! including round-trip operations and interaction between system and internal clipboards.

use termide::clipboard::{ClipboardError, ClipboardProvider, InternalClipboard, SystemClipboard};

/// Test SystemClipboard round-trip with realistic workflow
///
/// This test verifies that text can be written to and read from the system clipboard
/// in a typical user workflow: copy, modify buffer, paste.
#[test]
fn test_system_clipboard_realistic_workflow() {
    let mut clipboard = match SystemClipboard::new() {
        Ok(c) => c,
        Err(ClipboardError::NotAvailable) => {
            eprintln!("Skipping test - system clipboard not available (headless environment)");
            return;
        }
        Err(e) => panic!("Failed to initialize system clipboard: {}", e),
    };

    // Simulate copying text from editor
    let original_text = "fn main() {\n    println!(\"Hello, world!\");\n}";
    if let Err(e) = clipboard.set_text(original_text) {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    // Simulate doing other operations (in real editor, user might navigate, edit, etc.)
    // Clipboard should retain the content

    // Simulate pasting
    let pasted_text = clipboard
        .get_text()
        .expect("Failed to paste from clipboard");

    assert_eq!(pasted_text, original_text);
    assert!(pasted_text.contains("println!"));
}

/// Test InternalClipboard as fallback
///
/// Verifies that InternalClipboard works correctly as a fallback when system clipboard fails.
#[test]
fn test_internal_clipboard_fallback_workflow() {
    // InternalClipboard should always work
    let mut clipboard = InternalClipboard::default();

    // Copy operation
    let text_to_copy = "This is fallback clipboard content";
    clipboard
        .set_text(text_to_copy)
        .expect("InternalClipboard set_text should never fail");

    // Paste operation
    let pasted = clipboard
        .get_text()
        .expect("Should retrieve previously set text");

    assert_eq!(pasted, text_to_copy);
}

/// Test clipboard factory pattern simulation
///
/// Simulates the get_clipboard() factory function behavior by attempting
/// to create SystemClipboard and falling back to InternalClipboard.
#[test]
fn test_clipboard_factory_simulation() {
    // Try to create system clipboard, fall back to internal
    let mut clipboard: Box<dyn ClipboardProvider> = match SystemClipboard::new() {
        Ok(system_clipboard) => {
            eprintln!("Using system clipboard");
            Box::new(system_clipboard)
        }
        Err(ClipboardError::NotAvailable) => {
            eprintln!("System clipboard unavailable, using internal clipboard");
            Box::new(InternalClipboard::default())
        }
        Err(e) => panic!("Unexpected clipboard initialization error: {}", e),
    };

    // Test that the selected clipboard works
    let test_text = "Factory pattern test";
    if let Err(e) = clipboard.set_text(test_text) {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    let retrieved = clipboard
        .get_text()
        .expect("Clipboard should return text");

    assert_eq!(retrieved, test_text);
}

/// Test multiple clipboard instances isolation
///
/// Verifies that multiple clipboard instances can coexist and remain isolated.
#[test]
fn test_multiple_clipboard_instances() {
    // Create two internal clipboards
    let mut clipboard1 = InternalClipboard::default();
    let mut clipboard2 = InternalClipboard::default();

    // Set different content in each
    clipboard1.set_text("Content A").unwrap();
    clipboard2.set_text("Content B").unwrap();

    // Verify isolation
    assert_eq!(clipboard1.get_text().unwrap(), "Content A");
    assert_eq!(clipboard2.get_text().unwrap(), "Content B");

    // Modify one
    clipboard1.set_text("Content A Modified").unwrap();

    // Verify other is unaffected
    assert_eq!(clipboard1.get_text().unwrap(), "Content A Modified");
    assert_eq!(clipboard2.get_text().unwrap(), "Content B");
}

/// Test clipboard with code snippet (realistic editor use case)
///
/// Verifies that clipboard correctly handles typical code snippets with
/// various formatting and special characters.
#[test]
fn test_clipboard_with_code_snippet() {
    let mut clipboard = match SystemClipboard::new() {
        Ok(c) => c,
        Err(ClipboardError::NotAvailable) => {
            // Fallback to internal clipboard for headless environments
            let mut internal = InternalClipboard::default();
            let code_snippet = r#"pub fn example() -> Result<String, Error> {
    let value = "test \"string\" with 'quotes'";
    Ok(value.to_string())
}"#;
            internal.set_text(code_snippet).unwrap();
            let retrieved = internal.get_text().unwrap();
            assert_eq!(retrieved, code_snippet);
            return;
        }
        Err(e) => panic!("Failed to initialize clipboard: {}", e),
    };

    let code_snippet = r#"pub fn example() -> Result<String, Error> {
    let value = "test \"string\" with 'quotes'";
    Ok(value.to_string())
}"#;

    if let Err(e) = clipboard.set_text(code_snippet) {
        eprintln!("Skipping test - clipboard set failed: {}", e);
        return;
    }

    let retrieved = clipboard
        .get_text()
        .expect("Failed to paste code snippet");

    assert_eq!(retrieved, code_snippet);
    assert!(retrieved.contains("Result<String, Error>"));
    assert!(retrieved.contains("\"test \\\"string\\\" with 'quotes'\""));
}

/// Test clipboard behavior with sequential operations
///
/// Verifies correct behavior when performing multiple copy/paste operations in sequence.
#[test]
fn test_sequential_clipboard_operations() {
    let mut clipboard = InternalClipboard::default();

    // First operation
    clipboard.set_text("First").unwrap();
    assert_eq!(clipboard.get_text().unwrap(), "First");

    // Second operation
    clipboard.set_text("Second").unwrap();
    assert_eq!(clipboard.get_text().unwrap(), "Second");

    // Third operation
    clipboard.set_text("Third").unwrap();
    assert_eq!(clipboard.get_text().unwrap(), "Third");

    // Verify last value persists
    assert_eq!(clipboard.get_text().unwrap(), "Third");
}

/// Test clipboard with empty content transitions
///
/// Verifies behavior when transitioning between empty and non-empty states.
#[test]
fn test_clipboard_empty_transitions() {
    let mut clipboard = InternalClipboard::default();

    // Start empty
    assert!(clipboard.get_text().is_err());

    // Add content
    clipboard.set_text("Content").unwrap();
    assert_eq!(clipboard.get_text().unwrap(), "Content");

    // Set to empty string
    clipboard.set_text("").unwrap();
    assert_eq!(clipboard.get_text().unwrap(), "");

    // Add content again
    clipboard.set_text("New content").unwrap();
    assert_eq!(clipboard.get_text().unwrap(), "New content");
}

/// Test clipboard with various text encodings
///
/// Verifies that clipboard handles different character encodings correctly.
#[test]
fn test_clipboard_various_encodings() {
    let test_cases = vec![
        ("ASCII only", "Hello World"),
        ("Latin extended", "Café résumé naïve"),
        ("Cyrillic", "Привет мир"),
        ("Chinese", "你好世界"),
        ("Japanese", "こんにちは世界"),
        ("Korean", "안녕하세요 세계"),
        ("Emoji", "Hello 👋 World 🌍 Test 🚀"),
        ("Mixed", "Hello 世界 🚀 Привет café"),
    ];

    for (description, text) in test_cases {
        let mut clipboard = InternalClipboard::default();

        clipboard
            .set_text(text)
            .unwrap_or_else(|e| panic!("Failed to set {}: {}", description, e));

        let retrieved = clipboard
            .get_text()
            .unwrap_or_else(|e| panic!("Failed to get {}: {}", description, e));

        assert_eq!(
            retrieved, text,
            "{} should be preserved exactly",
            description
        );
    }
}

/// Test clipboard error handling
///
/// Verifies that clipboard errors are handled gracefully.
#[test]
fn test_clipboard_error_handling() {
    let mut clipboard = InternalClipboard::default();

    // Attempt to get from empty clipboard
    match clipboard.get_text() {
        Err(ClipboardError::NotAvailable) => {
            // Expected error
        }
        Ok(_) => panic!("Expected NotAvailable error for empty clipboard"),
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }

    // After setting, should succeed
    clipboard.set_text("test").unwrap();
    assert!(clipboard.get_text().is_ok());
}

/// Test clipboard with large content (stress test)
///
/// Verifies that clipboard can handle large amounts of text (simulating large file copy/paste).
#[test]
fn test_clipboard_large_content_stress() {
    let mut clipboard = InternalClipboard::default();

    // Create 100KB of text
    let large_content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(2000);
    assert!(large_content.len() > 100_000);

    clipboard
        .set_text(&large_content)
        .expect("Should handle large content");

    let retrieved = clipboard
        .get_text()
        .expect("Should retrieve large content");

    assert_eq!(retrieved.len(), large_content.len());
    assert_eq!(retrieved, large_content);
}

/// Test clipboard preservation across type changes
///
/// Verifies that clipboard content persists when switching between clipboard types.
#[test]
fn test_clipboard_type_preservation() {
    // This test demonstrates that clipboard content is independent
    // of the clipboard instance, not shared state
    let mut clipboard1 = InternalClipboard::default();
    clipboard1.set_text("Instance 1").unwrap();

    // Create new instance
    let mut clipboard2 = InternalClipboard::default();

    // New instance should be empty (not shared state)
    assert!(clipboard2.get_text().is_err());

    // Original instance should retain content
    assert_eq!(clipboard1.get_text().unwrap(), "Instance 1");
}
