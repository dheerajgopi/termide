//! Integration tests for mode transitions with keybinding system
//!
//! This test module validates that mode changes properly interact with the
//! keybinding system, particularly ensuring that:
//! - Sequence buffers are cleared on mode transitions
//! - Partial sequences don't persist across mode boundaries
//! - All mode transition paths work correctly
//! - Rapid mode switching doesn't leave stale state

use crate::editor::EditorMode;
use crate::input::bindings::register_default_bindings;
use crate::input::input_handler::{InputHandler, MatchResult};
use crate::input::keybinding::{
    BindingContext, KeyBinding, KeyPattern, KeySequence, Priority,
};
use crate::input::EditorCommand;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Helper to create a KeyEvent with no modifiers
fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Helper to create a character KeyEvent with no modifiers
fn char_key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

/// Helper to create a character KeyEvent with modifiers
fn char_key_mod(c: char, mods: KeyModifiers) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), mods)
}

/// Helper to create an InputHandler with default bindings and a test multi-key sequence
fn setup_handler_with_multikey() -> InputHandler {
    let mut handler = InputHandler::new();
    register_default_bindings(handler.registry_mut()).expect("defaults should register");

    // Register a test multi-key sequence "dd" for delete in Normal mode
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
    handler.register_binding(dd_binding).expect("dd should register");

    handler
}

#[cfg(test)]
mod partial_sequence_clearing {
    use super::*;

    #[test]
    fn test_partial_sequence_cleared_on_mode_change_normal_to_insert() {
        let mut handler = setup_handler_with_multikey();

        // Start "dd" sequence in Normal mode - should be Partial
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "First 'd' should be Partial in Normal mode"
        );

        // Simulate mode change to Insert (e.g., via 'i' command)
        handler.on_mode_change();

        // Type 'd' again in Insert mode - should be NoMatch (not completing the sequence)
        let result = handler.process_key_event(char_key('d'), EditorMode::Insert);
        assert_eq!(
            result,
            MatchResult::NoMatch,
            "Second 'd' in Insert mode should be NoMatch (buffer was cleared)"
        );
    }

    #[test]
    fn test_partial_sequence_cleared_on_mode_change_normal_to_prompt() {
        let mut handler = setup_handler_with_multikey();

        // Start "dd" sequence in Normal mode
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Simulate mode change to Prompt
        handler.on_mode_change();

        // Type 'd' in Prompt mode - should be NoMatch
        let result = handler.process_key_event(char_key('d'), EditorMode::Prompt);
        assert_eq!(
            result,
            MatchResult::NoMatch,
            "Second 'd' in Prompt mode should be NoMatch (buffer was cleared)"
        );
    }

    #[test]
    fn test_partial_sequence_cleared_on_esc_to_normal() {
        let mut handler = setup_handler_with_multikey();

        // Type 'd' in Normal mode
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Press Esc (which triggers mode change to Normal)
        let _result = handler.process_key_event(key(KeyCode::Esc), EditorMode::Normal);
        // Esc in Normal mode might be NoMatch or trigger some command, but buffer should clear

        // Simulate explicit mode change (in case Esc doesn't trigger it)
        handler.on_mode_change();

        // Type 'd' again - should start fresh, not complete the sequence
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "First 'd' after mode change should be Partial (new sequence)"
        );
    }

    #[test]
    fn test_insert_then_esc_clears_buffer() {
        let mut handler = setup_handler_with_multikey();

        // In Insert mode, type 'd' (NoMatch - will be inserted)
        let result = handler.process_key_event(char_key('d'), EditorMode::Insert);
        assert_eq!(result, MatchResult::NoMatch);

        // Press Esc to return to Normal mode
        let result = handler.process_key_event(key(KeyCode::Esc), EditorMode::Insert);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Normal)),
            "Esc in Insert mode should change to Normal"
        );

        // Call on_mode_change (simulating what main.rs does)
        handler.on_mode_change();

        // Now type 'd' in Normal mode - should be Partial (starting fresh)
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "First 'd' in Normal mode after Esc should be Partial"
        );

        // Type 'd' again - should complete the sequence
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::DeleteChar),
            "Second 'd' should complete 'dd' sequence"
        );
    }
}

#[cfg(test)]
mod all_mode_transition_paths {
    use super::*;

    #[test]
    fn test_insert_to_normal_via_esc() {
        let mut handler = InputHandler::new();
        register_default_bindings(handler.registry_mut()).unwrap();

        // Press Esc in Insert mode
        let result = handler.process_key_event(key(KeyCode::Esc), EditorMode::Insert);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Normal)),
            "Esc in Insert should change to Normal"
        );
    }

    #[test]
    fn test_normal_to_insert_via_i() {
        let mut handler = InputHandler::new();
        register_default_bindings(handler.registry_mut()).unwrap();

        // Press 'i' in Normal mode
        let result = handler.process_key_event(char_key('i'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Insert)),
            "'i' in Normal should change to Insert"
        );
    }

    #[test]
    fn test_prompt_to_previous_via_esc() {
        let mut handler = InputHandler::new();
        register_default_bindings(handler.registry_mut()).unwrap();

        // Press Esc in Prompt mode
        let result = handler.process_key_event(key(KeyCode::Esc), EditorMode::Prompt);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::CancelPrompt),
            "Esc in Prompt should cancel prompt"
        );
    }

    #[test]
    fn test_prompt_to_previous_via_enter() {
        let mut handler = InputHandler::new();
        register_default_bindings(handler.registry_mut()).unwrap();

        // Press Enter in Prompt mode
        let result = handler.process_key_event(key(KeyCode::Enter), EditorMode::Prompt);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::AcceptPrompt),
            "Enter in Prompt should accept prompt"
        );
    }

    #[test]
    fn test_all_transitions_clear_buffer() {
        let mut handler = setup_handler_with_multikey();

        // Start a partial sequence in Normal
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Transition to Insert
        handler.on_mode_change();
        let result = handler.process_key_event(char_key('d'), EditorMode::Insert);
        assert_eq!(result, MatchResult::NoMatch, "Buffer cleared on Normal->Insert");

        // Transition back to Normal
        handler.on_mode_change();

        // Start partial sequence again
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Transition to Prompt
        handler.on_mode_change();
        let result = handler.process_key_event(char_key('d'), EditorMode::Prompt);
        assert_eq!(result, MatchResult::NoMatch, "Buffer cleared on Normal->Prompt");

        // Transition back to Normal
        handler.on_mode_change();

        // Verify fresh start
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial, "Fresh sequence after Prompt->Normal");
    }
}

#[cfg(test)]
mod rapid_mode_switching {
    use super::*;

    #[test]
    fn test_rapid_normal_insert_normal() {
        let mut handler = setup_handler_with_multikey();

        // Normal mode: start 'd'
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Switch to Insert
        handler.on_mode_change();

        // Immediately switch back to Normal
        handler.on_mode_change();

        // Type 'd' - should be fresh Partial, not completing previous
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "Rapid mode switching should clear buffer"
        );
    }

    #[test]
    fn test_rapid_multiple_mode_changes() {
        let mut handler = setup_handler_with_multikey();

        // Start partial in Normal
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Rapid mode changes: Normal -> Insert -> Normal -> Prompt -> Normal
        handler.on_mode_change(); // Normal -> Insert
        handler.on_mode_change(); // Insert -> Normal
        handler.on_mode_change(); // Normal -> Prompt
        handler.on_mode_change(); // Prompt -> Normal

        // Buffer should be completely clear
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "Multiple rapid mode changes should clear buffer"
        );

        // Complete the sequence properly
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Matched(EditorCommand::DeleteChar));
    }

    #[test]
    fn test_no_cross_contamination() {
        let mut handler = setup_handler_with_multikey();

        // Type 'd' in Insert mode (should be NoMatch)
        let result = handler.process_key_event(char_key('d'), EditorMode::Insert);
        assert_eq!(result, MatchResult::NoMatch);

        // Switch to Normal mode
        handler.on_mode_change();

        // Type 'd' once (Partial)
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Switch to Insert
        handler.on_mode_change();

        // Type 'd' (NoMatch)
        let result = handler.process_key_event(char_key('d'), EditorMode::Insert);
        assert_eq!(result, MatchResult::NoMatch);

        // Switch back to Normal
        handler.on_mode_change();

        // Type 'd' - should be fresh Partial
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "No cross-contamination between modes"
        );
    }
}

#[cfg(test)]
mod timeout_during_mode_transition {
    use super::*;

    #[test]
    fn test_mode_change_during_timeout_period() {
        let mut handler = InputHandler::with_timeout(Duration::from_millis(500));
        register_default_bindings(handler.registry_mut()).unwrap();

        // Register multi-key sequence
        let dd_binding = KeyBinding::new(
            KeySequence::new(vec![
                KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
                KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
            ])
            .unwrap(),
            EditorCommand::DeleteChar,
            BindingContext::Mode(EditorMode::Normal),
            Priority::Default,
        );
        handler.register_binding(dd_binding).unwrap();

        // Start sequence
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Mode change before timeout (should clear immediately)
        handler.on_mode_change();

        // Even if we check timeout, buffer should stay clear
        let timed_out = handler.check_timeout();
        assert!(
            !timed_out,
            "Buffer was already cleared by mode change, so no timeout"
        );

        // Type 'd' in new mode - should be fresh
        let result = handler.process_key_event(char_key('d'), EditorMode::Insert);
        assert_eq!(result, MatchResult::NoMatch);
    }

    #[test]
    fn test_mode_change_clears_before_timeout_fires() {
        let mut handler = InputHandler::with_timeout(Duration::from_secs(10));
        register_default_bindings(handler.registry_mut()).unwrap();

        // Register multi-key sequence
        let dd_binding = KeyBinding::new(
            KeySequence::new(vec![
                KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
                KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
            ])
            .unwrap(),
            EditorCommand::DeleteChar,
            BindingContext::Mode(EditorMode::Normal),
            Priority::Default,
        );
        handler.register_binding(dd_binding).unwrap();

        // Start sequence (with long timeout)
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Immediately change mode (before timeout)
        handler.on_mode_change();

        // Buffer should be clear immediately, not waiting for timeout
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "Buffer cleared immediately on mode change, not after timeout"
        );
    }
}

#[cfg(test)]
mod mode_specific_behavior {
    use super::*;

    #[test]
    fn test_same_key_different_modes() {
        let mut handler = InputHandler::new();
        register_default_bindings(handler.registry_mut()).unwrap();

        // Enter in Insert mode - inserts newline
        let result = handler.process_key_event(key(KeyCode::Enter), EditorMode::Insert);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::InsertChar('\n')),
            "Enter in Insert mode inserts newline"
        );

        // Enter in Prompt mode - accepts prompt
        let result = handler.process_key_event(key(KeyCode::Enter), EditorMode::Prompt);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::AcceptPrompt),
            "Enter in Prompt mode accepts prompt"
        );

        // Enter in Normal mode - NoMatch (not bound)
        let result = handler.process_key_event(key(KeyCode::Enter), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::NoMatch,
            "Enter in Normal mode is not bound"
        );
    }

    #[test]
    fn test_esc_behavior_across_modes() {
        let mut handler = InputHandler::new();
        register_default_bindings(handler.registry_mut()).unwrap();

        // Esc in Insert mode - change to Normal
        let result = handler.process_key_event(key(KeyCode::Esc), EditorMode::Insert);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Normal)),
            "Esc in Insert changes to Normal"
        );

        // Esc in Prompt mode - cancel prompt
        let result = handler.process_key_event(key(KeyCode::Esc), EditorMode::Prompt);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::CancelPrompt),
            "Esc in Prompt cancels prompt"
        );

        // Esc in Normal mode - NoMatch (not bound by default)
        let result = handler.process_key_event(key(KeyCode::Esc), EditorMode::Normal);
        // Could be NoMatch or some command depending on bindings
        // Just verify it doesn't panic or error
        assert!(
            matches!(result, MatchResult::NoMatch | MatchResult::Matched(_)),
            "Esc in Normal mode should work without error"
        );
    }

    #[test]
    fn test_backspace_behavior_across_modes() {
        let mut handler = InputHandler::new();
        register_default_bindings(handler.registry_mut()).unwrap();

        // Backspace in Insert mode - delete char
        let result = handler.process_key_event(key(KeyCode::Backspace), EditorMode::Insert);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::DeleteChar),
            "Backspace in Insert deletes char"
        );

        // Backspace in Prompt mode - delete from prompt
        let result = handler.process_key_event(key(KeyCode::Backspace), EditorMode::Prompt);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::PromptDeleteChar),
            "Backspace in Prompt deletes from prompt"
        );

        // Backspace in Normal mode - NoMatch (not bound)
        let result = handler.process_key_event(key(KeyCode::Backspace), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::NoMatch,
            "Backspace in Normal is not bound"
        );
    }
}

#[cfg(test)]
mod integration_with_global_shortcuts {
    use super::*;

    #[test]
    fn test_global_shortcuts_work_after_mode_change() {
        let mut handler = InputHandler::new();
        register_default_bindings(handler.registry_mut()).unwrap();

        // Ctrl+S in Insert mode
        let result = handler.process_key_event(
            char_key_mod('s', KeyModifiers::CONTROL),
            EditorMode::Insert,
        );
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::Save),
            "Ctrl+S saves in Insert mode"
        );

        // Switch to Normal
        handler.on_mode_change();

        // Ctrl+S in Normal mode should still work
        let result = handler.process_key_event(
            char_key_mod('s', KeyModifiers::CONTROL),
            EditorMode::Normal,
        );
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::Save),
            "Ctrl+S saves in Normal mode"
        );
    }

    #[test]
    fn test_mode_change_doesnt_affect_global_shortcuts() {
        let mut handler = setup_handler_with_multikey();

        // Start partial sequence
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Mode change clears buffer
        handler.on_mode_change();

        // Global shortcut should still work
        let result = handler.process_key_event(
            char_key_mod('q', KeyModifiers::CONTROL),
            EditorMode::Normal,
        );
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::Quit),
            "Global shortcuts unaffected by buffer clearing"
        );
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_multiple_on_mode_change_calls() {
        let mut handler = setup_handler_with_multikey();

        // Start partial sequence
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Multiple on_mode_change calls (shouldn't cause issues)
        handler.on_mode_change();
        handler.on_mode_change();
        handler.on_mode_change();

        // Should still work normally
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);
    }

    #[test]
    fn test_on_mode_change_with_empty_buffer() {
        let mut handler = InputHandler::new();
        register_default_bindings(handler.registry_mut()).unwrap();

        // Call on_mode_change with no keys pressed (empty buffer)
        handler.on_mode_change();

        // Should not cause any issues
        let result = handler.process_key_event(char_key('i'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Insert)));
    }

    #[test]
    fn test_mode_change_after_complete_match() {
        let mut handler = setup_handler_with_multikey();

        // Complete a sequence
        handler.process_key_event(char_key('d'), EditorMode::Normal);
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Matched(EditorCommand::DeleteChar));

        // Buffer should already be clear, but on_mode_change shouldn't cause issues
        handler.on_mode_change();

        // Should work normally
        let result = handler.process_key_event(char_key('i'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Insert)));
    }
}
