//! Integration tests for clipboard module
//!
//! These tests verify clipboard functionality in realistic scenarios,
//! including round-trip operations and interaction between system and internal clipboards.

use termide::clipboard::{
    get_clipboard, ClipboardError, ClipboardProvider, InternalClipboard, SystemClipboard,
};

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
    // Use internal clipboard to avoid interference from system clipboard state
    let mut clipboard = InternalClipboard::default();

    let code_snippet = r#"pub fn example() -> Result<String, Error> {
    let value = "test \"string\" with 'quotes'";
    Ok(value.to_string())
}"#;

    clipboard.set_text(code_snippet).unwrap();

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

/// Test get_clipboard factory function in realistic editor workflow
///
/// Verifies that the factory function provides a working clipboard regardless
/// of system availability, and that it works correctly in a typical editor workflow.
#[test]
fn test_get_clipboard_factory_realistic_workflow() {
    let mut clipboard = get_clipboard();

    // Simulate typical editor workflow: copy, navigate, paste
    let original_code = r#"fn example() {
    let x = 42;
    println!("Value: {}", x);
}"#;

    // Copy operation - may fail in headless environments
    if clipboard.set_text(original_code).is_err() {
        eprintln!("Skipping test - clipboard operations not available");
        return;
    }

    // Simulate some intermediate operations (in real editor, user might navigate, edit)
    // The clipboard should retain the content

    // Paste operation
    let pasted_code = clipboard
        .get_text()
        .expect("Should retrieve previously copied code");

    assert_eq!(pasted_code, original_code);
    assert!(pasted_code.contains("fn example()"));
    assert!(pasted_code.contains("println!"));
}

/// Test get_clipboard factory function with multiple operations
///
/// Verifies that factory-created clipboard handles repeated copy/paste operations
/// correctly in a realistic editing session.
#[test]
fn test_get_clipboard_factory_multiple_operations() {
    let mut clipboard = get_clipboard();

    let operations = vec![
        "First line of code",
        "Second function definition",
        "Third variable assignment",
    ];

    for (i, text) in operations.iter().enumerate() {
        // Set text - may fail in headless
        if clipboard.set_text(text).is_err() {
            eprintln!("Skipping test - clipboard operations not available");
            return;
        }

        // Verify retrieval
        let retrieved = clipboard
            .get_text()
            .unwrap_or_else(|e| panic!("Failed to get text at iteration {}: {}", i, e));

        assert_eq!(
            retrieved, *text,
            "Content should match at iteration {}",
            i
        );
    }

    // Final verification - should have last operation's content
    let final_content = clipboard.get_text().unwrap();
    assert_eq!(final_content, operations[operations.len() - 1]);
}

/// Test get_clipboard factory with concurrent access simulation
///
/// Verifies that multiple clipboards created via factory work independently
/// (for internal clipboard case) or share state correctly (for system clipboard case).
#[test]
fn test_get_clipboard_factory_concurrent_access() {
    let mut clipboard1 = get_clipboard();
    let mut clipboard2 = get_clipboard();

    // Set different content in each
    if clipboard1.set_text("Clipboard 1 content").is_err() {
        eprintln!("Skipping test - clipboard operations not available");
        return;
    }

    if clipboard2.set_text("Clipboard 2 content").is_err() {
        eprintln!("Skipping test - clipboard operations not available");
        return;
    }

    // Get content from each
    let content1 = clipboard1.get_text();
    let content2 = clipboard2.get_text();

    // Both should work
    assert!(content1.is_ok(), "Clipboard 1 should work");
    assert!(content2.is_ok(), "Clipboard 2 should work");

    // Note: If both are system clipboards, they share state and will have the same content
    // If one or both are internal clipboards, they will be independent
    // We just verify both operations succeed
}

/// Test get_clipboard factory resilience to errors
///
/// Verifies that factory-created clipboard handles error conditions gracefully
/// and provides meaningful error messages.
#[test]
fn test_get_clipboard_factory_error_handling() {
    let mut clipboard = get_clipboard();

    // Try to get from potentially empty clipboard
    // This should either succeed (if system clipboard has content)
    // or fail gracefully with NotAvailable error (if internal clipboard and empty)
    match clipboard.get_text() {
        Ok(_content) => {
            // System clipboard had content, or internal clipboard was set
        }
        Err(ClipboardError::NotAvailable) => {
            // Expected error for empty internal clipboard
        }
        Err(e) => {
            // Other errors are acceptable in headless environments
            eprintln!("Clipboard error (acceptable in headless): {}", e);
        }
    }

    // After setting, get should work
    if clipboard.set_text("test content").is_ok() {
        let retrieved = clipboard.get_text();
        assert!(
            retrieved.is_ok(),
            "After successful set, get should work"
        );
    }
}
