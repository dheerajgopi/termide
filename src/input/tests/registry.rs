//! Unit tests for KeyBindingRegistry

use crate::editor::EditorMode;
use crate::input::keybinding::{
    BindingContext, KeyBinding, KeyPattern, KeySequence, Priority,
};
use crate::input::registry::{BindingError, KeyBindingRegistry};
use crate::input::EditorCommand;
use crossterm::event::{KeyCode, KeyModifiers};
use std::time::Duration;

/// Helper function to create a simple binding for testing
fn create_binding(
    key: char,
    modifiers: KeyModifiers,
    command: EditorCommand,
    context: BindingContext,
    priority: Priority,
) -> KeyBinding {
    KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char(key), modifiers)])
            .expect("single char is valid"),
        command,
        context,
        priority,
    )
}

/// Helper function to create a two-key sequence binding
fn create_sequence_binding(
    key1: char,
    key2: char,
    command: EditorCommand,
    context: BindingContext,
    priority: Priority,
) -> KeyBinding {
    KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char(key1), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char(key2), KeyModifiers::NONE),
        ])
        .expect("sequence is valid"),
        command,
        context,
        priority,
    )
}

#[test]
fn test_registry_new() {
    let registry = KeyBindingRegistry::new(Duration::from_secs(1));
    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
}

#[test]
fn test_registry_default() {
    let registry = KeyBindingRegistry::default();
    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
}

#[test]
fn test_register_single_binding() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let result = registry.register(binding);
    assert!(result.is_ok());
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
}

#[test]
fn test_register_multiple_bindings() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let binding1 = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let binding2 = create_binding(
        'a',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    registry.register(binding1).expect("first binding");
    registry.register(binding2).expect("second binding");

    assert_eq!(registry.len(), 2);
}

#[test]
fn test_register_conflict_same_priority() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let binding1 = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let binding2 = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::Save, // Different command, same sequence+context+priority
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    registry.register(binding1).expect("first registration");
    let result = registry.register(binding2);

    assert!(matches!(result, Err(BindingError::Conflict { .. })));
    assert_eq!(registry.len(), 1); // Only first binding registered
}

#[test]
fn test_register_override_different_priority() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let default_binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let user_binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::Save, // Different command, higher priority
        BindingContext::Mode(EditorMode::Normal),
        Priority::User,
    );

    registry.register(default_binding).expect("default");
    let result = registry.register(user_binding);

    assert!(result.is_ok()); // Higher priority can shadow lower priority
    assert_eq!(registry.len(), 2); // Both bindings registered
}

#[test]
fn test_register_different_contexts_no_conflict() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let normal_binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let insert_binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Normal),
        BindingContext::Mode(EditorMode::Insert),
        Priority::Default,
    );

    registry.register(normal_binding).expect("normal");
    let result = registry.register(insert_binding);

    assert!(result.is_ok()); // Different contexts = no conflict
    assert_eq!(registry.len(), 2);
}

#[test]
fn test_register_different_sequences_no_conflict() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let binding1 = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let binding2 = create_binding(
        'a',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    registry.register(binding1).expect("i binding");
    let result = registry.register(binding2);

    assert!(result.is_ok()); // Different sequences = no conflict
    assert_eq!(registry.len(), 2);
}

#[test]
fn test_register_multi_key_sequence() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let binding = create_sequence_binding(
        'd',
        'd',
        EditorCommand::Save,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let result = registry.register(binding);
    assert!(result.is_ok());
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_unregister_existing_binding() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let sequence = KeySequence::new(vec![KeyPattern::new(
        KeyCode::Char('i'),
        KeyModifiers::NONE,
    )])
    .expect("i is valid");

    let binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    registry.register(binding).expect("registration");
    assert_eq!(registry.len(), 1);

    registry.unregister(&sequence, &BindingContext::Mode(EditorMode::Normal));
    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
}

#[test]
fn test_unregister_nonexistent_binding() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let sequence = KeySequence::new(vec![KeyPattern::new(
        KeyCode::Char('i'),
        KeyModifiers::NONE,
    )])
    .expect("i is valid");

    // Unregistering nonexistent binding is a no-op
    registry.unregister(&sequence, &BindingContext::Mode(EditorMode::Normal));
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_unregister_with_wrong_context() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let sequence = KeySequence::new(vec![KeyPattern::new(
        KeyCode::Char('i'),
        KeyModifiers::NONE,
    )])
    .expect("i is valid");

    let binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    registry.register(binding).expect("registration");
    assert_eq!(registry.len(), 1);

    // Unregister with wrong context doesn't remove the binding
    registry.unregister(&sequence, &BindingContext::Mode(EditorMode::Insert));
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_unregister_keeps_other_bindings() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let binding1 = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let binding2 = create_binding(
        'a',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    registry.register(binding1).expect("i binding");
    registry.register(binding2).expect("a binding");
    assert_eq!(registry.len(), 2);

    let sequence = KeySequence::new(vec![KeyPattern::new(
        KeyCode::Char('i'),
        KeyModifiers::NONE,
    )])
    .expect("i is valid");

    registry.unregister(&sequence, &BindingContext::Mode(EditorMode::Normal));
    assert_eq!(registry.len(), 1); // Only 'i' removed, 'a' remains
}

#[test]
fn test_clear_sequence() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Clear on empty registry is safe
    registry.clear_sequence();

    // Add some bindings
    let binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    registry.register(binding).expect("registration");

    // Clear sequence doesn't affect bindings
    registry.clear_sequence();
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_global_context_binding() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let binding = create_binding(
        's',
        KeyModifiers::CONTROL,
        EditorCommand::Save,
        BindingContext::Global,
        Priority::Default,
    );

    let result = registry.register(binding);
    assert!(result.is_ok());
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_plugin_context_binding() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(
            KeyCode::Char('l'),
            KeyModifiers::NONE,
        )])
        .expect("l is valid"),
        EditorCommand::Save,
        BindingContext::Plugin {
            name: "lsp".to_string(),
            modes: Some(vec![EditorMode::Normal]),
        },
        Priority::Plugin,
    );

    let result = registry.register(binding);
    assert!(result.is_ok());
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_multiple_priorities_can_coexist() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register the same sequence+context at different priorities
    let default = create_binding(
        's',
        KeyModifiers::CONTROL,
        EditorCommand::Save,
        BindingContext::Global,
        Priority::Default,
    );

    let plugin = create_binding(
        's',
        KeyModifiers::CONTROL,
        EditorCommand::Save,
        BindingContext::Global,
        Priority::Plugin,
    );

    let user = create_binding(
        's',
        KeyModifiers::CONTROL,
        EditorCommand::Save,
        BindingContext::Global,
        Priority::User,
    );

    registry.register(default).expect("default");
    registry.register(plugin).expect("plugin");
    registry.register(user).expect("user");

    assert_eq!(registry.len(), 3);
    // Priority ordering will be validated through matching tests in T004
}
