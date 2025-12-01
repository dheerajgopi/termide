//! Unit tests for InputHandler

use crate::editor::EditorMode;
use crate::input::input_handler::{InputHandler, MatchResult};
use crate::input::keybinding::{BindingContext, KeyBinding, KeyPattern, KeySequence, Priority};
use crate::input::EditorCommand;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::thread;
use std::time::Duration;

// Helper function to create a KeyEvent
fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

#[test]
fn test_register_binding() {
    let mut handler = InputHandler::new();

    let binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
            .expect("i is valid"),
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let result = handler.register_binding(binding);
    assert!(result.is_ok());
}

#[test]
fn test_register_duplicate_binding_conflict() {
    let mut handler = InputHandler::new();

    let binding1 = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
            .expect("i is valid"),
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    let binding2 = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
            .expect("i is valid"),
        EditorCommand::ChangeMode(EditorMode::Insert), // Same everything
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    handler.register_binding(binding1).expect("first should succeed");
    let result = handler.register_binding(binding2);
    assert!(result.is_err()); // Conflict
}

#[test]
fn test_single_key_matched() {
    let mut handler = InputHandler::new();

    // Register 'i' to enter insert mode in Normal mode
    let binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
            .expect("i is valid"),
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler.register_binding(binding).unwrap();

    // Press 'i' in Normal mode
    let event = key_event(KeyCode::Char('i'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);

    match result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Insert));
        }
        _ => panic!("Expected Matched, got {:?}", result),
    }
}

#[test]
fn test_single_key_no_match_wrong_mode() {
    let mut handler = InputHandler::new();

    // Register 'i' only for Normal mode
    let binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
            .expect("i is valid"),
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler.register_binding(binding).unwrap();

    // Press 'i' in Insert mode (wrong mode)
    let event = key_event(KeyCode::Char('i'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert_eq!(result, MatchResult::NoMatch);
}

#[test]
fn test_single_key_no_match_unmapped() {
    let mut handler = InputHandler::new();

    // Don't register any bindings
    // Press 'x' (unmapped)
    let event = key_event(KeyCode::Char('x'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);

    assert_eq!(result, MatchResult::NoMatch);
}

#[test]
fn test_multi_key_sequence_partial_then_matched() {
    let mut handler = InputHandler::new();

    // Register "dd" for delete line
    let dd_binding = KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        ])
        .expect("dd is valid"),
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler.register_binding(dd_binding).unwrap();

    // First 'd' - partial match
    let event = key_event(KeyCode::Char('d'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::Partial);

    // Second 'd' - complete match
    let event = key_event(KeyCode::Char('d'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    match result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::DeleteChar);
        }
        _ => panic!("Expected Matched, got {:?}", result),
    }
}

#[test]
fn test_multi_key_sequence_partial_then_no_match() {
    let mut handler = InputHandler::new();

    // Register "dd" for delete line
    let dd_binding = KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        ])
        .expect("dd is valid"),
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler.register_binding(dd_binding).unwrap();

    // First 'd' - partial match
    let event = key_event(KeyCode::Char('d'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::Partial);

    // Press 'x' instead of second 'd' - no match
    let event = key_event(KeyCode::Char('x'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::NoMatch);
}

#[test]
fn test_three_key_sequence() {
    let mut handler = InputHandler::new();

    // Register "abc" sequence
    let abc_binding = KeyBinding::new(
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
    handler.register_binding(abc_binding).unwrap();

    // 'a' - partial
    let event = key_event(KeyCode::Char('a'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::Partial);

    // 'b' - still partial
    let event = key_event(KeyCode::Char('b'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::Partial);

    // 'c' - complete match
    let event = key_event(KeyCode::Char('c'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    match result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::Save);
        }
        _ => panic!("Expected Matched, got {:?}", result),
    }
}

#[test]
fn test_on_mode_change_clears_buffer() {
    let mut handler = InputHandler::new();

    // Register "dd" for Normal mode
    let dd_binding = KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        ])
        .expect("dd is valid"),
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler.register_binding(dd_binding).unwrap();

    // Type 'd' - partial match
    let event = key_event(KeyCode::Char('d'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::Partial);

    // Mode change - buffer should be cleared
    handler.on_mode_change();

    // Type 'd' again in Insert mode - should be NoMatch (not continuing sequence)
    let result = handler.process_key_event(event, EditorMode::Insert);
    assert_eq!(result, MatchResult::NoMatch);
}

#[test]
fn test_global_context_active_in_normal() {
    let mut handler = InputHandler::new();

    // Register Ctrl+S as global (should work in Normal and Insert, not Prompt)
    let save_binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(
            KeyCode::Char('s'),
            KeyModifiers::CONTROL,
        )])
        .expect("Ctrl+S is valid"),
        EditorCommand::Save,
        BindingContext::Global,
        Priority::Default,
    );
    handler.register_binding(save_binding).unwrap();

    // Press Ctrl+S in Normal mode
    let event = key_event(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Normal);
    match result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::Save);
        }
        _ => panic!("Expected Matched, got {:?}", result),
    }
}

#[test]
fn test_global_context_active_in_insert() {
    let mut handler = InputHandler::new();

    // Register Ctrl+S as global
    let save_binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(
            KeyCode::Char('s'),
            KeyModifiers::CONTROL,
        )])
        .expect("Ctrl+S is valid"),
        EditorCommand::Save,
        BindingContext::Global,
        Priority::Default,
    );
    handler.register_binding(save_binding).unwrap();

    // Press Ctrl+S in Insert mode
    let event = key_event(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Insert);
    match result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::Save);
        }
        _ => panic!("Expected Matched, got {:?}", result),
    }
}

#[test]
fn test_global_context_not_active_in_prompt() {
    let mut handler = InputHandler::new();

    // Register Ctrl+S as global
    let save_binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(
            KeyCode::Char('s'),
            KeyModifiers::CONTROL,
        )])
        .expect("Ctrl+S is valid"),
        EditorCommand::Save,
        BindingContext::Global,
        Priority::Default,
    );
    handler.register_binding(save_binding).unwrap();

    // Press Ctrl+S in Prompt mode - should not match
    let event = key_event(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Prompt);
    assert_eq!(result, MatchResult::NoMatch);
}

#[test]
fn test_priority_ordering_user_over_default() {
    let mut handler = InputHandler::new();

    // Register default binding for 'i'
    let default_binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
            .expect("i is valid"),
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    // Register user override for 'i' (different command)
    let user_binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
            .expect("i is valid"),
        EditorCommand::Save, // Different command
        BindingContext::Mode(EditorMode::Normal),
        Priority::User,
    );

    handler.register_binding(default_binding).unwrap();
    handler.register_binding(user_binding).unwrap();

    // Press 'i' - should execute user binding (Save), not default (ChangeMode)
    let event = key_event(KeyCode::Char('i'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    match result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::Save); // User binding wins
        }
        _ => panic!("Expected Matched, got {:?}", result),
    }
}

#[test]
fn test_key_event_to_pattern_conversion() {
    let mut handler = InputHandler::new();

    // Register binding for lowercase 'a' with no modifiers
    let binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('a'), KeyModifiers::NONE)])
            .expect("a is valid"),
        EditorCommand::Save,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler.register_binding(binding).unwrap();

    // Test exact match: 'a' with NONE
    let event = key_event(KeyCode::Char('a'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert!(matches!(result, MatchResult::Matched(_)));
}

#[test]
fn test_key_event_modifier_mismatch() {
    let mut handler = InputHandler::new();

    // Register binding for 'a' with CONTROL
    let binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(
            KeyCode::Char('a'),
            KeyModifiers::CONTROL,
        )])
        .expect("Ctrl+A is valid"),
        EditorCommand::Save,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler.register_binding(binding).unwrap();

    // Press 'a' with SHIFT (wrong modifier)
    let event = key_event(KeyCode::Char('a'), KeyModifiers::SHIFT);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::NoMatch);

    // Press 'a' with NONE (no modifier, wrong)
    let event = key_event(KeyCode::Char('a'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::NoMatch);

    // Press 'a' with CONTROL (correct)
    let event = key_event(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert!(matches!(result, MatchResult::Matched(_)));
}

#[test]
fn test_check_timeout_clears_buffer() {
    let mut handler = InputHandler::with_timeout(Duration::from_millis(100));

    // Register "dd" sequence
    let dd_binding = KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        ])
        .expect("dd is valid"),
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler.register_binding(dd_binding).unwrap();

    // Type 'd' - partial match
    let event = key_event(KeyCode::Char('d'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::Partial);

    // Check timeout immediately - should not clear
    assert!(!handler.check_timeout());

    // Wait for timeout
    thread::sleep(Duration::from_millis(150));

    // Check timeout - should clear buffer
    assert!(handler.check_timeout());

    // Type 'd' again - should be partial, not completing the old sequence
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::Partial);
}

#[test]
fn test_match_clears_buffer() {
    let mut handler = InputHandler::new();

    // Register single key 'i'
    let binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
            .expect("i is valid"),
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler.register_binding(binding).unwrap();

    // Press 'i' - matched
    let event = key_event(KeyCode::Char('i'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert!(matches!(result, MatchResult::Matched(_)));

    // Buffer should be cleared, so pressing 'i' again should be a fresh match
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert!(matches!(result, MatchResult::Matched(_)));
}

#[test]
fn test_no_match_clears_buffer() {
    let mut handler = InputHandler::new();

    // Register "dd" sequence
    let dd_binding = KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        ])
        .expect("dd is valid"),
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler.register_binding(dd_binding).unwrap();

    // Type 'd' - partial
    let event = key_event(KeyCode::Char('d'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::Partial);

    // Type 'x' - no match, buffer should be cleared
    let event = key_event(KeyCode::Char('x'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::NoMatch);

    // Type 'd' again - should be partial (fresh start)
    let event = key_event(KeyCode::Char('d'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    assert_eq!(result, MatchResult::Partial);
}

#[test]
fn test_multiple_bindings_different_contexts() {
    let mut handler = InputHandler::new();

    // Register 'i' for Normal mode (enter insert)
    let normal_binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
            .expect("i is valid"),
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    // Register 'i' for Insert mode (just insert character - normally handled by default)
    let insert_binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
            .expect("i is valid"),
        EditorCommand::InsertChar('i'),
        BindingContext::Mode(EditorMode::Insert),
        Priority::Default,
    );

    handler.register_binding(normal_binding).unwrap();
    handler.register_binding(insert_binding).unwrap();

    // Press 'i' in Normal mode
    let event = key_event(KeyCode::Char('i'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);
    match result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Insert));
        }
        _ => panic!("Expected Matched in Normal mode"),
    }

    // Press 'i' in Insert mode
    let result = handler.process_key_event(event, EditorMode::Insert);
    match result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::InsertChar('i'));
        }
        _ => panic!("Expected Matched in Insert mode"),
    }
}

#[test]
fn test_integration_full_workflow() {
    let mut handler = InputHandler::new();

    // Register multiple bindings
    // 1. 'i' to enter insert mode (Normal mode)
    handler
        .register_binding(KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)])
                .expect("i is valid"),
            EditorCommand::ChangeMode(EditorMode::Insert),
            BindingContext::Mode(EditorMode::Normal),
            Priority::Default,
        ))
        .unwrap();

    // 2. "dd" to delete line (Normal mode)
    handler
        .register_binding(KeyBinding::new(
            KeySequence::new(vec![
                KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
                KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
            ])
            .expect("dd is valid"),
            EditorCommand::DeleteChar,
            BindingContext::Mode(EditorMode::Normal),
            Priority::Default,
        ))
        .unwrap();

    // 3. Ctrl+S to save (Global)
    handler
        .register_binding(KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(
                KeyCode::Char('s'),
                KeyModifiers::CONTROL,
            )])
            .expect("Ctrl+S is valid"),
            EditorCommand::Save,
            BindingContext::Global,
            Priority::Default,
        ))
        .unwrap();

    // Workflow: Start in Normal mode
    // Step 1: Press 'i' to enter insert mode
    let result = handler.process_key_event(
        key_event(KeyCode::Char('i'), KeyModifiers::NONE),
        EditorMode::Normal,
    );
    assert!(matches!(result, MatchResult::Matched(_)));

    // Step 2: Switch to Insert mode, press Ctrl+S to save
    let result = handler.process_key_event(
        key_event(KeyCode::Char('s'), KeyModifiers::CONTROL),
        EditorMode::Insert,
    );
    match result {
        MatchResult::Matched(cmd) => assert_eq!(cmd, EditorCommand::Save),
        _ => panic!("Expected Save command"),
    }

    // Step 3: Switch back to Normal mode
    handler.on_mode_change();

    // Step 4: Type "dd" to delete line
    let result = handler.process_key_event(
        key_event(KeyCode::Char('d'), KeyModifiers::NONE),
        EditorMode::Normal,
    );
    assert_eq!(result, MatchResult::Partial);

    let result = handler.process_key_event(
        key_event(KeyCode::Char('d'), KeyModifiers::NONE),
        EditorMode::Normal,
    );
    match result {
        MatchResult::Matched(cmd) => assert_eq!(cmd, EditorCommand::DeleteChar),
        _ => panic!("Expected DeleteChar command"),
    }
}
