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
    // Priority ordering will be validated through matching tests below
}

// ============================================================================
// Sequence Matching Tests
// ============================================================================

#[test]
fn test_add_to_sequence() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let pattern1 = KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE);
    let pattern2 = KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE);

    registry.add_to_sequence(pattern1);
    registry.add_to_sequence(pattern2);

    // Buffer should contain both patterns (verified through matching tests)
}

#[test]
fn test_find_match_single_key() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register 'i' to enter insert mode in Normal mode
    let binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    // No match before adding to sequence
    assert_eq!(registry.find_match(EditorMode::Normal), None);

    // Add 'i' to sequence
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE));

    // Should match in Normal mode
    let result = registry.find_match(EditorMode::Normal);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), &EditorCommand::ChangeMode(EditorMode::Insert));

    // Should not match in Insert mode (wrong context)
    assert_eq!(registry.find_match(EditorMode::Insert), None);
}

#[test]
fn test_find_match_multi_key_sequence() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register "dd" for delete in Normal mode
    let binding = create_sequence_binding(
        'd',
        'd',
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    // Add first 'd' - no match yet
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    assert_eq!(registry.find_match(EditorMode::Normal), None);

    // Add second 'd' - now we have a match
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    let result = registry.find_match(EditorMode::Normal);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), &EditorCommand::DeleteChar);
}

#[test]
fn test_find_match_three_key_sequence() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register "abc" sequence
    let binding = KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('a'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('b'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('c'), KeyModifiers::NONE),
        ])
        .expect("abc is valid"),
        EditorCommand::Save,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    // Add first key - no match
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('a'), KeyModifiers::NONE));
    assert_eq!(registry.find_match(EditorMode::Normal), None);

    // Add second key - no match
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('b'), KeyModifiers::NONE));
    assert_eq!(registry.find_match(EditorMode::Normal), None);

    // Add third key - match!
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('c'), KeyModifiers::NONE));
    let result = registry.find_match(EditorMode::Normal);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), &EditorCommand::Save);
}

#[test]
fn test_find_match_context_filtering() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register 'i' only for Normal mode
    let binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE));

    // Should match in Normal mode
    assert!(registry.find_match(EditorMode::Normal).is_some());

    // Should not match in Insert mode (context not active)
    assert_eq!(registry.find_match(EditorMode::Insert), None);

    // Should not match in Prompt mode (context not active)
    assert_eq!(registry.find_match(EditorMode::Prompt), None);
}

#[test]
fn test_find_match_global_context() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register Ctrl+S as global save
    let binding = create_binding(
        's',
        KeyModifiers::CONTROL,
        EditorCommand::Save,
        BindingContext::Global,
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    registry.add_to_sequence(KeyPattern::new(
        KeyCode::Char('s'),
        KeyModifiers::CONTROL,
    ));

    // Should match in Normal mode
    assert!(registry.find_match(EditorMode::Normal).is_some());

    // Should match in Insert mode
    assert!(registry.find_match(EditorMode::Insert).is_some());

    // Should NOT match in Prompt mode (Global excludes Prompt)
    assert_eq!(registry.find_match(EditorMode::Prompt), None);
}

#[test]
fn test_find_match_priority_ordering() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register same sequence at different priorities
    let default = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let user = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::Save, // Different command
        BindingContext::Mode(EditorMode::Normal),
        Priority::User,
    );

    registry.register(default).expect("default");
    registry.register(user).expect("user");

    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE));

    // Should return User priority binding (higher priority)
    let result = registry.find_match(EditorMode::Normal);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), &EditorCommand::Save);
}

#[test]
fn test_find_match_empty_buffer() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    // Empty buffer should not match
    assert_eq!(registry.find_match(EditorMode::Normal), None);
}

#[test]
fn test_is_partial_match_two_key_sequence() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register "dd" sequence
    let binding = create_sequence_binding(
        'd',
        'd',
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    // Empty buffer - no partial match
    assert!(!registry.is_partial_match(EditorMode::Normal));

    // Add first 'd' - this is a partial match
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    assert!(registry.is_partial_match(EditorMode::Normal));

    // Add second 'd' - no longer partial (it's complete)
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    assert!(!registry.is_partial_match(EditorMode::Normal));
}

#[test]
fn test_is_partial_match_three_key_sequence() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register "abc" sequence
    let binding = KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('a'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('b'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('c'), KeyModifiers::NONE),
        ])
        .expect("abc is valid"),
        EditorCommand::Save,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    // First key - partial match
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('a'), KeyModifiers::NONE));
    assert!(registry.is_partial_match(EditorMode::Normal));

    // Second key - still partial
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('b'), KeyModifiers::NONE));
    assert!(registry.is_partial_match(EditorMode::Normal));

    // Third key - complete (not partial)
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('c'), KeyModifiers::NONE));
    assert!(!registry.is_partial_match(EditorMode::Normal));
}

#[test]
fn test_is_partial_match_wrong_sequence() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register "dd" sequence
    let binding = create_sequence_binding(
        'd',
        'd',
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    // Add wrong first key - no partial match
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('x'), KeyModifiers::NONE));
    assert!(!registry.is_partial_match(EditorMode::Normal));
}

#[test]
fn test_is_partial_match_context_filtering() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register "dd" only for Normal mode
    let binding = create_sequence_binding(
        'd',
        'd',
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));

    // Should be partial match in Normal mode
    assert!(registry.is_partial_match(EditorMode::Normal));

    // Should NOT be partial match in Insert mode (context not active)
    assert!(!registry.is_partial_match(EditorMode::Insert));
}

#[test]
fn test_check_timeout_empty_buffer() {
    let mut registry = KeyBindingRegistry::new(Duration::from_millis(100));

    // Empty buffer should return false
    assert!(!registry.check_timeout());
}

#[test]
fn test_check_timeout_before_expiry() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(10));

    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));

    // Check immediately - should not timeout
    assert!(!registry.check_timeout());
}

#[test]
fn test_check_timeout_after_expiry() {
    use std::thread;

    let mut registry = KeyBindingRegistry::new(Duration::from_millis(50));

    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));

    // Wait for timeout
    thread::sleep(Duration::from_millis(100));

    // Should timeout and clear buffer
    assert!(registry.check_timeout());

    // Second check should return false (buffer already empty)
    assert!(!registry.check_timeout());
}

#[test]
fn test_check_timeout_resets_on_new_key() {
    use std::thread;

    let mut registry = KeyBindingRegistry::new(Duration::from_millis(100));

    // Add first key
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));

    // Wait half the timeout
    thread::sleep(Duration::from_millis(50));

    // Add second key - this resets the timeout
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));

    // Check immediately after second key - should not timeout
    assert!(!registry.check_timeout());

    // Wait for original timeout period
    thread::sleep(Duration::from_millis(60));

    // Should still not timeout (timer was reset)
    assert!(!registry.check_timeout());
}

#[test]
fn test_sequence_cleared_after_match() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register "dd" sequence
    let binding = create_sequence_binding(
        'd',
        'd',
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    registry.register(binding).expect("registration");

    // Add first 'd' - partial match
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    assert!(registry.is_partial_match(EditorMode::Normal));

    // Clear buffer
    registry.clear_sequence();

    // No longer a partial match (buffer is empty)
    assert!(!registry.is_partial_match(EditorMode::Normal));
}

#[test]
fn test_integration_complete_sequence_workflow() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register "dd" for delete line
    let dd_binding = create_sequence_binding(
        'd',
        'd',
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    // Register "dw" for delete word
    let dw_binding = KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('w'), KeyModifiers::NONE),
        ])
        .expect("dw is valid"),
        EditorCommand::Save,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    registry.register(dd_binding).expect("dd");
    registry.register(dw_binding).expect("dw");

    // Step 1: Add 'd'
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    assert_eq!(registry.find_match(EditorMode::Normal), None); // No complete match
    assert!(registry.is_partial_match(EditorMode::Normal)); // Partial match exists

    // Step 2: Add 'd' (completing "dd")
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    let result = registry.find_match(EditorMode::Normal);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), &EditorCommand::DeleteChar);
    assert!(!registry.is_partial_match(EditorMode::Normal)); // Complete, not partial

    // Step 3: Clear and try "dw"
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    assert!(registry.is_partial_match(EditorMode::Normal));

    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('w'), KeyModifiers::NONE));
    let result = registry.find_match(EditorMode::Normal);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), &EditorCommand::Save);
}

#[test]
fn test_user_priority_overrides_default_priority() {
    use crate::input::keybinding::Priority;
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    
    // Register default Ctrl+S -> Save
    let default_binding = create_binding(
        's',
        KeyModifiers::CONTROL,
        EditorCommand::Save,
        BindingContext::Global,
        Priority::Default,
    );
    
    // Register user Ctrl+S -> Quit (higher priority)
    let user_binding = create_binding(
        's',
        KeyModifiers::CONTROL,
        EditorCommand::Quit,
        BindingContext::Global,
        Priority::User,
    );
    
    registry.register(default_binding).expect("default");
    registry.register(user_binding).expect("user");
    
    // Add Ctrl+S to sequence buffer
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('s'), KeyModifiers::CONTROL));

    // Find match - should return User priority binding (Quit)
    let cmd = registry.find_match(EditorMode::Normal).expect("Should find match");
    assert_eq!(cmd, &EditorCommand::Quit, "User priority should override Default");
}

// ==================== Tests for unregister_by_priority ====================

#[test]
fn test_unregister_by_priority_removes_only_target_priority() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register bindings at different priorities
    let default_binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let plugin_binding = create_binding(
        'j',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Down),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Plugin,
    );

    let user_binding = create_binding(
        'k',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Up),
        BindingContext::Mode(EditorMode::Normal),
        Priority::User,
    );

    registry.register(default_binding).unwrap();
    registry.register(plugin_binding).unwrap();
    registry.register(user_binding).unwrap();

    assert_eq!(registry.len(), 3);

    // Remove only User priority bindings
    let removed = registry.unregister_by_priority(Priority::User);
    assert_eq!(removed, 1);
    assert_eq!(registry.len(), 2);

    // Verify User binding is gone but others remain
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert!(registry.find_match(EditorMode::Normal).is_none(), "User binding should be removed");

    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('j'), KeyModifiers::NONE));
    assert!(registry.find_match(EditorMode::Normal).is_some(), "Plugin binding should remain");

    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE));
    assert!(registry.find_match(EditorMode::Normal).is_some(), "Default binding should remain");
}

#[test]
fn test_unregister_by_priority_multiple_bindings() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register multiple User priority bindings
    let user_binding1 = create_binding(
        'a',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Left),
        BindingContext::Mode(EditorMode::Normal),
        Priority::User,
    );

    let user_binding2 = create_binding(
        'b',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Right),
        BindingContext::Mode(EditorMode::Normal),
        Priority::User,
    );

    let user_binding3 = create_binding(
        'c',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Up),
        BindingContext::Mode(EditorMode::Normal),
        Priority::User,
    );

    let default_binding = create_binding(
        'd',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Down),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    registry.register(user_binding1).unwrap();
    registry.register(user_binding2).unwrap();
    registry.register(user_binding3).unwrap();
    registry.register(default_binding).unwrap();

    assert_eq!(registry.len(), 4);

    // Remove all User priority bindings
    let removed = registry.unregister_by_priority(Priority::User);
    assert_eq!(removed, 3);
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_unregister_by_priority_empty_registry() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Unregistering from empty registry should return 0
    let removed = registry.unregister_by_priority(Priority::User);
    assert_eq!(removed, 0);
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_unregister_by_priority_no_matching_priority() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register only Default bindings
    let default_binding = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    registry.register(default_binding).unwrap();
    assert_eq!(registry.len(), 1);

    // Try to remove User priority - should remove nothing
    let removed = registry.unregister_by_priority(Priority::User);
    assert_eq!(removed, 0);
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_unregister_by_priority_preserves_priority_ordering() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register bindings in mixed order
    let default_binding = create_binding(
        'a',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Left),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let user_binding = create_binding(
        'b',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Right),
        BindingContext::Mode(EditorMode::Normal),
        Priority::User,
    );

    let plugin_binding = create_binding(
        'c',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Up),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Plugin,
    );

    registry.register(default_binding).unwrap();
    registry.register(user_binding).unwrap();
    registry.register(plugin_binding).unwrap();

    // Remove User priority
    registry.unregister_by_priority(Priority::User);

    // Register another User binding - it should be inserted in correct priority position
    let new_user_binding = create_binding(
        'd',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Down),
        BindingContext::Mode(EditorMode::Normal),
        Priority::User,
    );

    registry.register(new_user_binding).unwrap();
    assert_eq!(registry.len(), 3);
}

#[test]
fn test_unregister_by_priority_with_different_contexts() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register User bindings in different contexts
    let user_binding_normal = create_binding(
        'i',
        KeyModifiers::NONE,
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::User,
    );

    let user_binding_insert = create_binding(
        'a',
        KeyModifiers::CONTROL,
        EditorCommand::MoveToLineStart,
        BindingContext::Mode(EditorMode::Insert),
        Priority::User,
    );

    let user_binding_global = create_binding(
        's',
        KeyModifiers::CONTROL,
        EditorCommand::Save,
        BindingContext::Global,
        Priority::User,
    );

    registry.register(user_binding_normal).unwrap();
    registry.register(user_binding_insert).unwrap();
    registry.register(user_binding_global).unwrap();

    assert_eq!(registry.len(), 3);

    // Remove all User bindings regardless of context
    let removed = registry.unregister_by_priority(Priority::User);
    assert_eq!(removed, 3);
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_unregister_by_priority_all_priorities() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register bindings at all priority levels
    let default_binding = create_binding(
        'a',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Left),
        BindingContext::Global,
        Priority::Default,
    );

    let plugin_binding = create_binding(
        'b',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Right),
        BindingContext::Global,
        Priority::Plugin,
    );

    let user_binding = create_binding(
        'c',
        KeyModifiers::NONE,
        EditorCommand::MoveCursor(crate::input::Direction::Up),
        BindingContext::Global,
        Priority::User,
    );

    registry.register(default_binding).unwrap();
    registry.register(plugin_binding).unwrap();
    registry.register(user_binding).unwrap();

    // Remove each priority level one by one
    let removed_default = registry.unregister_by_priority(Priority::Default);
    assert_eq!(removed_default, 1);
    assert_eq!(registry.len(), 2);

    let removed_plugin = registry.unregister_by_priority(Priority::Plugin);
    assert_eq!(removed_plugin, 1);
    assert_eq!(registry.len(), 1);

    let removed_user = registry.unregister_by_priority(Priority::User);
    assert_eq!(removed_user, 1);
    assert_eq!(registry.len(), 0);
}
