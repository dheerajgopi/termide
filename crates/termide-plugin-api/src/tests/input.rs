//! Unit tests for plugin input API

use crate::input::*;

// ============================================================================
// EditorMode Tests
// ============================================================================

#[test]
fn test_mode_parsing_case_insensitive() {
    assert_eq!(EditorMode::from_str("insert").unwrap(), EditorMode::Insert);
    assert_eq!(EditorMode::from_str("INSERT").unwrap(), EditorMode::Insert);
    assert_eq!(EditorMode::from_str("Normal").unwrap(), EditorMode::Normal);
    assert_eq!(EditorMode::from_str("PROMPT").unwrap(), EditorMode::Prompt);
}

#[test]
fn test_mode_parsing_whitespace_trimming() {
    assert_eq!(
        EditorMode::from_str(" prompt ").unwrap(),
        EditorMode::Prompt
    );
    assert_eq!(
        EditorMode::from_str("\tinsert\n").unwrap(),
        EditorMode::Insert
    );
}

#[test]
fn test_mode_parsing_rejects_invalid() {
    assert!(EditorMode::from_str("invalid").is_err());
    assert!(EditorMode::from_str("").is_err());
    assert!(EditorMode::from_str("insrt").is_err());
}

// ============================================================================
// PluginBindingBuilder Tests
// ============================================================================

#[test]
fn test_builder_auto_namespacing() {
    // Command without dot gets namespaced
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+K", "show_info")
        .build()
        .unwrap();
    assert_eq!(binding.command, "my-plugin.show_info");

    // Command with dot is preserved as-is
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+K", "other-plugin.command")
        .build()
        .unwrap();
    assert_eq!(binding.command, "other-plugin.command");

    // Multiple dots preserved
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+K", "lsp.rust.format")
        .build()
        .unwrap();
    assert_eq!(binding.command, "lsp.rust.format");
}

#[test]
fn test_builder_mode_specific_binding() {
    let binding = PluginBindingBuilder::new("test")
        .bind("g d", "goto_def")
        .in_mode("normal")
        .build()
        .unwrap();

    assert_eq!(binding.context, BindingContext::Mode(EditorMode::Normal));

    // Case-insensitive mode name
    let binding = PluginBindingBuilder::new("test")
        .bind("g d", "goto_def")
        .in_mode("INSERT")
        .build()
        .unwrap();

    assert_eq!(binding.context, BindingContext::Mode(EditorMode::Insert));
}

#[test]
fn test_builder_multi_mode_binding() {
    let binding = PluginBindingBuilder::new("test")
        .bind("Ctrl+/", "comment")
        .in_modes(&["insert", "normal"])
        .build()
        .unwrap();

    match binding.context {
        BindingContext::Modes(modes) => {
            assert_eq!(modes.len(), 2);
            assert!(modes.contains(&EditorMode::Insert));
            assert!(modes.contains(&EditorMode::Normal));
        }
        _ => panic!("Expected Modes context"),
    }
}

#[test]
fn test_builder_multi_mode_with_case_variations() {
    let binding = PluginBindingBuilder::new("test")
        .bind("Ctrl+/", "comment")
        .in_modes(&["INSERT", "Normal", "prompt"])
        .build()
        .unwrap();

    match binding.context {
        BindingContext::Modes(modes) => {
            assert_eq!(modes.len(), 3);
            assert!(modes.contains(&EditorMode::Insert));
            assert!(modes.contains(&EditorMode::Normal));
            assert!(modes.contains(&EditorMode::Prompt));
        }
        _ => panic!("Expected Modes context"),
    }
}

#[test]
fn test_builder_global_is_default() {
    let binding_default = PluginBindingBuilder::new("test")
        .bind("Ctrl+S", "save")
        .build()
        .unwrap();

    let binding_explicit = PluginBindingBuilder::new("test")
        .bind("Ctrl+S", "save")
        .global()
        .build()
        .unwrap();

    assert_eq!(binding_default.context, BindingContext::Global);
    assert_eq!(binding_explicit.context, BindingContext::Global);
}

#[test]
fn test_builder_mode_overrides_previous_context() {
    // Last context method called wins
    let binding = PluginBindingBuilder::new("test")
        .bind("Ctrl+K", "info")
        .in_modes(&["insert", "normal"])
        .in_mode("prompt") // This overrides the previous in_modes
        .build()
        .unwrap();

    assert_eq!(binding.context, BindingContext::Mode(EditorMode::Prompt));
}

// ============================================================================
// Builder Validation Tests
// ============================================================================

#[test]
fn test_builder_validation_missing_bind() {
    let result = PluginBindingBuilder::new("test").global().build();

    assert!(result.is_err());
    match result.unwrap_err() {
        BindingError::BuilderValidation(msg) => {
            assert!(msg.contains("sequence is required"));
        }
        _ => panic!("Expected BuilderValidation error"),
    }
}

#[test]
fn test_builder_validation_empty_sequence() {
    let result = PluginBindingBuilder::new("test")
        .bind("", "command")
        .build();

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BindingError::EmptySequence));
}

#[test]
fn test_builder_validation_whitespace_only_sequence() {
    let result = PluginBindingBuilder::new("test")
        .bind("   ", "command")
        .build();

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BindingError::EmptySequence));
}

#[test]
fn test_builder_validation_empty_command() {
    let result = PluginBindingBuilder::new("test")
        .bind("Ctrl+S", "")
        .build();

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BindingError::EmptyCommand));
}

#[test]
fn test_builder_validation_whitespace_only_command() {
    let result = PluginBindingBuilder::new("test")
        .bind("Ctrl+S", "   ")
        .build();

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BindingError::EmptyCommand));
}

// ============================================================================
// Complex Builder Scenarios
// ============================================================================

#[test]
fn test_builder_complex_multi_key_sequence() {
    let binding = PluginBindingBuilder::new("vim-plugin")
        .bind("g g", "goto_top")
        .in_mode("normal")
        .build()
        .unwrap();

    assert_eq!(binding.sequence, "g g");
    assert_eq!(binding.command, "vim-plugin.goto_top");
    assert_eq!(binding.context, BindingContext::Mode(EditorMode::Normal));
}

#[test]
fn test_builder_with_modifiers_and_special_keys() {
    let binding = PluginBindingBuilder::new("editor")
        .bind("Ctrl+Shift+F", "format_document")
        .global()
        .build()
        .unwrap();

    assert_eq!(binding.sequence, "Ctrl+Shift+F");
    assert_eq!(binding.command, "editor.format_document");
}

#[test]
fn test_builder_emacs_style_sequence() {
    let binding = PluginBindingBuilder::new("emacs")
        .bind("Ctrl+X k", "kill_buffer")
        .build()
        .unwrap();

    assert_eq!(binding.sequence, "Ctrl+X k");
    assert_eq!(binding.command, "emacs.kill_buffer");
}

// ============================================================================
// Error Display Tests
// ============================================================================

#[test]
fn test_error_messages_are_descriptive() {
    let err = BindingError::EmptySequence;
    assert_eq!(err.to_string(), "key sequence cannot be empty");

    let err = BindingError::EmptyCommand;
    assert_eq!(err.to_string(), "command cannot be empty");

    let err = BindingError::InvalidMode("insrt".to_string());
    let msg = err.to_string();
    assert!(msg.contains("insrt"));
    assert!(msg.contains("insert"));
    assert!(msg.contains("normal"));
    assert!(msg.contains("prompt"));
}

#[test]
fn test_conflict_error_contains_all_details() {
    let err = BindingError::Conflict {
        sequence: "Ctrl+K".to_string(),
        existing_command: "show_docs".to_string(),
        plugin: "lsp".to_string(),
    };

    let msg = err.to_string();
    assert!(msg.contains("Ctrl+K"));
    assert!(msg.contains("show_docs"));
    assert!(msg.contains("lsp"));
}
