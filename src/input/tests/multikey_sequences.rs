//! Integration tests for multi-key sequences with timeout behavior
//!
//! This test module validates that multi-key sequences work correctly in the
//! integrated system, particularly ensuring that:
//! - Multi-key sequences complete successfully when typed within timeout
//! - Partial sequences time out and clear after the configured duration
//! - Rapid typing within timeout window works correctly
//! - Invalid key combinations properly clear the buffer
//! - Timeout countdown resets on each matching key
//! - Configurable timeout values work as expected

use crate::editor::EditorMode;
use crate::input::bindings::register_default_bindings;
use crate::input::input_handler::{InputHandler, MatchResult};
use crate::input::keybinding::{
    BindingContext, KeyBinding, KeyPattern, KeySequence, Priority,
};
use crate::input::{Direction, EditorCommand};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::thread;
use std::time::Duration;

/// Helper to create a character KeyEvent with no modifiers
fn char_key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

/// Helper to create an InputHandler with default bindings and test multi-key sequences
fn setup_handler_with_sequences(timeout: Duration) -> InputHandler {
    let mut handler = InputHandler::with_timeout(timeout);
    register_default_bindings(handler.registry_mut()).expect("defaults should register");

    // Register "dd" for delete line in Normal mode
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
    handler
        .register_binding(dd_binding)
        .expect("dd should register");

    // Register "gg" for go to top in Normal mode
    let gg_binding = KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
        ])
        .expect("gg is valid"),
        EditorCommand::MoveCursor(Direction::Up), // Using Up as placeholder for "go to top"
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler
        .register_binding(gg_binding)
        .expect("gg should register");

    handler
}

/// Helper to setup a handler with a 3-key sequence
fn setup_handler_with_three_key_sequence(timeout: Duration) -> InputHandler {
    let mut handler = setup_handler_with_sequences(timeout);

    // Register "daw" (delete a word) in Normal mode - 3-key sequence
    let daw_binding = KeyBinding::new(
        KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('a'), KeyModifiers::NONE),
            KeyPattern::new(KeyCode::Char('w'), KeyModifiers::NONE),
        ])
        .expect("daw is valid"),
        EditorCommand::Save, // Using Save as placeholder for "delete a word" (testing purposes)
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );
    handler
        .register_binding(daw_binding)
        .expect("daw should register");

    handler
}

#[cfg(test)]
mod successful_sequence_completion {
    use super::*;

    #[test]
    fn test_dd_sequence_completes_within_timeout() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Press 'd' - should be Partial
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "First 'd' should be Partial match"
        );

        // Press 'd' again within timeout - should complete
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::DeleteChar),
            "Second 'd' should complete 'dd' sequence"
        );
    }

    #[test]
    fn test_gg_sequence_completes_within_timeout() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Press 'g' - should be Partial
        let result = handler.process_key_event(char_key('g'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial, "First 'g' should be Partial");

        // Press 'g' again within timeout - should complete
        let result = handler.process_key_event(char_key('g'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::MoveCursor(Direction::Up)),
            "Second 'g' should complete 'gg' sequence"
        );
    }

    #[test]
    fn test_three_key_sequence_completes() {
        let mut handler = setup_handler_with_three_key_sequence(Duration::from_secs(1));

        // Press 'd' - Partial (could be 'dd' or 'daw')
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Press 'a' - still Partial (continuing 'daw')
        let result = handler.process_key_event(char_key('a'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Press 'w' - completes 'daw'
        let result = handler.process_key_event(char_key('w'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::Save),
            "Third key should complete 'daw' sequence"
        );
    }

    #[test]
    fn test_rapid_sequence_completion() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Type 'dd' very rapidly (no delays)
        let result1 = handler.process_key_event(char_key('d'), EditorMode::Normal);
        let result2 = handler.process_key_event(char_key('d'), EditorMode::Normal);

        assert_eq!(result1, MatchResult::Partial);
        assert_eq!(result2, MatchResult::Matched(EditorCommand::DeleteChar));
    }

    #[test]
    fn test_multiple_sequences_in_succession() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Complete 'dd' sequence
        handler.process_key_event(char_key('d'), EditorMode::Normal);
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Matched(EditorCommand::DeleteChar));

        // Complete 'gg' sequence
        handler.process_key_event(char_key('g'), EditorMode::Normal);
        let result = handler.process_key_event(char_key('g'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::MoveCursor(Direction::Up))
        );

        // Complete 'dd' again
        handler.process_key_event(char_key('d'), EditorMode::Normal);
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Matched(EditorCommand::DeleteChar));
    }
}

#[cfg(test)]
mod timeout_behavior {
    use super::*;

    #[test]
    fn test_partial_sequence_times_out() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(100));

        // Press 'd' - Partial
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Wait for timeout
        thread::sleep(Duration::from_millis(150));

        // Check timeout - should clear buffer
        let timed_out = handler.check_timeout();
        assert!(timed_out, "Buffer should be cleared due to timeout");

        // Press 'd' again - should be Partial (starting fresh sequence)
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "After timeout, 'd' should start fresh sequence"
        );
    }

    #[test]
    fn test_timeout_prevents_late_completion() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(100));

        // Press 'd'
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait for timeout
        thread::sleep(Duration::from_millis(150));
        handler.check_timeout();

        // Press 'd' again - should NOT complete 'dd'
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "Second 'd' after timeout should not complete sequence"
        );
    }

    #[test]
    fn test_no_timeout_when_buffer_empty() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(100));

        // Don't press any keys
        thread::sleep(Duration::from_millis(150));

        // Check timeout with empty buffer - should return false
        let timed_out = handler.check_timeout();
        assert!(
            !timed_out,
            "Timeout check should return false when buffer is empty"
        );
    }

    #[test]
    fn test_timeout_after_complete_match() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(100));

        // Complete 'dd' sequence
        handler.process_key_event(char_key('d'), EditorMode::Normal);
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait
        thread::sleep(Duration::from_millis(150));

        // Check timeout - buffer already cleared by match, should return false
        let timed_out = handler.check_timeout();
        assert!(
            !timed_out,
            "Timeout check should return false when buffer was already cleared"
        );
    }

    #[test]
    fn test_multiple_timeout_checks() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(100));

        // Press 'd'
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // First check before timeout - no clear
        let timed_out = handler.check_timeout();
        assert!(!timed_out, "Should not timeout immediately");

        // Wait for timeout
        thread::sleep(Duration::from_millis(150));

        // Second check after timeout - should clear
        let timed_out = handler.check_timeout();
        assert!(timed_out, "Should timeout after waiting");

        // Third check - buffer already empty
        let timed_out = handler.check_timeout();
        assert!(!timed_out, "Should not timeout again, buffer already empty");
    }
}

#[cfg(test)]
mod invalid_key_combinations {
    use super::*;

    #[test]
    fn test_invalid_second_key_clears_buffer() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Press 'd' - Partial
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Press 'x' (not a valid continuation) - should be NoMatch
        let result = handler.process_key_event(char_key('x'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::NoMatch,
            "Invalid continuation should be NoMatch"
        );

        // Press 'd' again - should start fresh Partial
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "After invalid key, 'd' should start fresh"
        );
    }

    #[test]
    fn test_partial_then_single_key_binding() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Press 'd' - Partial
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Press 'i' (single-key binding for insert mode) - should be NoMatch first (clears buffer)
        let result = handler.process_key_event(char_key('i'), EditorMode::Normal);
        // The 'di' sequence doesn't match anything, so buffer clears and 'i' is NoMatch
        assert_eq!(result, MatchResult::NoMatch);

        // Now press 'i' again - should match ChangeMode(Insert)
        let result = handler.process_key_event(char_key('i'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Insert))
        );
    }

    #[test]
    fn test_interleaved_sequences() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Start 'dd': press 'd'
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Switch to 'gg': press 'g' - invalid continuation, clears buffer
        let result = handler.process_key_event(char_key('g'), EditorMode::Normal);
        assert_eq!(result, MatchResult::NoMatch);

        // Now start fresh 'gg': press 'g'
        let result = handler.process_key_event(char_key('g'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);

        // Complete 'gg': press 'g'
        let result = handler.process_key_event(char_key('g'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::MoveCursor(Direction::Up))
        );
    }

    #[test]
    fn test_three_key_sequence_invalid_middle_key() {
        let mut handler = setup_handler_with_three_key_sequence(Duration::from_secs(1));

        // Press 'd' - Partial
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Press 'x' (invalid) - NoMatch
        let result = handler.process_key_event(char_key('x'), EditorMode::Normal);
        assert_eq!(result, MatchResult::NoMatch);

        // Buffer should be clear, start fresh
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);
    }
}

#[cfg(test)]
mod timeout_reset_behavior {
    use super::*;

    #[test]
    fn test_timeout_resets_on_each_key() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(200));

        // Press 'd' at T=0
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait 150ms (less than timeout)
        thread::sleep(Duration::from_millis(150));

        // Press 'd' at T=150ms - should reset timeout
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::DeleteChar),
            "Sequence should complete even though first key was 150ms ago"
        );
    }

    #[test]
    fn test_three_key_sequence_timeout_reset() {
        let mut handler = setup_handler_with_three_key_sequence(Duration::from_millis(200));

        // Press 'd' at T=0
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait 150ms
        thread::sleep(Duration::from_millis(150));

        // Press 'a' at T=150ms - resets timeout
        handler.process_key_event(char_key('a'), EditorMode::Normal);

        // Wait another 150ms (total 300ms from first key, but only 150ms from second)
        thread::sleep(Duration::from_millis(150));

        // Press 'w' at T=300ms - should complete because timeout reset at 'a'
        let result = handler.process_key_event(char_key('w'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Matched(EditorCommand::Save),
            "Sequence should complete with timeout reset on each key"
        );
    }

    #[test]
    fn test_timeout_doesnt_reset_on_invalid_key() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(200));

        // Press 'd' - Partial
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait 150ms
        thread::sleep(Duration::from_millis(150));

        // Press 'x' (invalid) - clears buffer
        handler.process_key_event(char_key('x'), EditorMode::Normal);

        // The timeout was NOT reset, buffer was cleared. Start fresh
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "After invalid key, new sequence should start fresh"
        );
    }
}

#[cfg(test)]
mod configurable_timeout {
    use super::*;

    #[test]
    fn test_timeout_500ms() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(500));

        // Press 'd'
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait 400ms (less than timeout)
        thread::sleep(Duration::from_millis(400));

        // Should not have timed out yet
        let timed_out = handler.check_timeout();
        assert!(!timed_out, "Should not timeout before 500ms");

        // Press 'd' - should complete
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Matched(EditorCommand::DeleteChar));
    }

    #[test]
    fn test_timeout_1000ms() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(1000));

        // Press 'd'
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait 900ms (less than timeout)
        thread::sleep(Duration::from_millis(900));

        // Should not have timed out yet
        let timed_out = handler.check_timeout();
        assert!(!timed_out, "Should not timeout before 1000ms");

        // Press 'd' - should complete
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Matched(EditorCommand::DeleteChar));
    }

    #[test]
    fn test_timeout_2000ms() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(2000));

        // Press 'd'
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait 1800ms (less than timeout)
        thread::sleep(Duration::from_millis(1800));

        // Should not have timed out yet
        let timed_out = handler.check_timeout();
        assert!(!timed_out, "Should not timeout before 2000ms");

        // Press 'd' - should complete
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Matched(EditorCommand::DeleteChar));
    }

    #[test]
    fn test_very_short_timeout() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(50));

        // Press 'd'
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait 100ms (more than timeout)
        thread::sleep(Duration::from_millis(100));

        // Should have timed out
        let timed_out = handler.check_timeout();
        assert!(timed_out, "Should timeout after 50ms");

        // Press 'd' - should be fresh Partial, not completing previous
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Partial);
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_rapid_typing_stress() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Type 10 'dd' sequences rapidly
        for _ in 0..10 {
            let result1 = handler.process_key_event(char_key('d'), EditorMode::Normal);
            let result2 = handler.process_key_event(char_key('d'), EditorMode::Normal);

            assert_eq!(result1, MatchResult::Partial);
            assert_eq!(result2, MatchResult::Matched(EditorCommand::DeleteChar));
        }
    }

    #[test]
    fn test_alternating_sequences_stress() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Alternate between 'dd' and 'gg' 10 times
        for i in 0..10 {
            if i % 2 == 0 {
                handler.process_key_event(char_key('d'), EditorMode::Normal);
                let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
                assert_eq!(result, MatchResult::Matched(EditorCommand::DeleteChar));
            } else {
                handler.process_key_event(char_key('g'), EditorMode::Normal);
                let result = handler.process_key_event(char_key('g'), EditorMode::Normal);
                assert_eq!(
                    result,
                    MatchResult::Matched(EditorCommand::MoveCursor(
                        Direction::Up
                    ))
                );
            }
        }
    }

    #[test]
    fn test_many_partial_sequences_with_timeout() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(50));

        // Create 20 partial sequences that timeout
        for _ in 0..20 {
            handler.process_key_event(char_key('d'), EditorMode::Normal);
            thread::sleep(Duration::from_millis(100));
            handler.check_timeout();
        }

        // Should still work normally after many timeouts
        let result1 = handler.process_key_event(char_key('d'), EditorMode::Normal);
        let result2 = handler.process_key_event(char_key('d'), EditorMode::Normal);

        assert_eq!(result1, MatchResult::Partial);
        assert_eq!(result2, MatchResult::Matched(EditorCommand::DeleteChar));
    }

    #[test]
    fn test_invalid_keys_stress() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Type many invalid sequences
        for c in ['d', 'x', 'g', 'z', 'd', 'q'].iter() {
            handler.process_key_event(char_key(*c), EditorMode::Normal);
        }

        // Should still work normally after invalid keys
        let result1 = handler.process_key_event(char_key('d'), EditorMode::Normal);
        let result2 = handler.process_key_event(char_key('d'), EditorMode::Normal);

        assert_eq!(result1, MatchResult::Partial);
        assert_eq!(result2, MatchResult::Matched(EditorCommand::DeleteChar));
    }
}

#[cfg(test)]
mod buffer_state_verification {
    use super::*;

    #[test]
    fn test_buffer_cleared_on_complete_match() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Complete 'dd'
        handler.process_key_event(char_key('d'), EditorMode::Normal);
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Verify buffer is clear by starting a new sequence
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "Buffer should be clear after complete match"
        );
    }

    #[test]
    fn test_buffer_cleared_on_no_match() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Start partial then invalid
        handler.process_key_event(char_key('d'), EditorMode::Normal);
        handler.process_key_event(char_key('x'), EditorMode::Normal); // NoMatch, clears buffer

        // Verify buffer is clear
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "Buffer should be clear after NoMatch"
        );
    }

    #[test]
    fn test_buffer_preserved_on_partial() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Press 'd' - Partial
        let result1 = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result1, MatchResult::Partial);

        // Press 'd' again - should complete (buffer was preserved)
        let result2 = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result2,
            MatchResult::Matched(EditorCommand::DeleteChar),
            "Buffer should be preserved on Partial"
        );
    }

    #[test]
    fn test_buffer_cleared_on_timeout() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(100));

        // Press 'd'
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait and timeout
        thread::sleep(Duration::from_millis(150));
        handler.check_timeout();

        // Verify buffer is clear
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(
            result,
            MatchResult::Partial,
            "Buffer should be clear after timeout"
        );
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_same_key_repeated_different_sequences() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // 'dd' sequence
        let result1 = handler.process_key_event(char_key('d'), EditorMode::Normal);
        let result2 = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result1, MatchResult::Partial);
        assert_eq!(result2, MatchResult::Matched(EditorCommand::DeleteChar));

        // 'gg' sequence
        let result1 = handler.process_key_event(char_key('g'), EditorMode::Normal);
        let result2 = handler.process_key_event(char_key('g'), EditorMode::Normal);
        assert_eq!(result1, MatchResult::Partial);
        assert_eq!(
            result2,
            MatchResult::Matched(EditorCommand::MoveCursor(Direction::Up))
        );
    }

    #[test]
    fn test_timeout_exactly_at_boundary() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(100));

        // Press 'd'
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Wait exactly 100ms
        thread::sleep(Duration::from_millis(100));

        // Should timeout (>= timeout duration)
        let timed_out = handler.check_timeout();
        assert!(
            timed_out,
            "Should timeout at exactly the timeout boundary"
        );
    }

    #[test]
    fn test_check_timeout_called_during_sequence() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Press 'd'
        handler.process_key_event(char_key('d'), EditorMode::Normal);

        // Check timeout immediately (shouldn't timeout)
        let timed_out = handler.check_timeout();
        assert!(!timed_out);

        // Complete sequence - should still work
        let result = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result, MatchResult::Matched(EditorCommand::DeleteChar));
    }

    #[test]
    fn test_empty_buffer_timeout_check() {
        let mut handler = setup_handler_with_sequences(Duration::from_millis(100));

        // Don't press any keys, just check timeout
        let timed_out = handler.check_timeout();
        assert!(!timed_out, "Empty buffer should not report timeout");
    }

    #[test]
    fn test_partial_after_global_shortcut() {
        let mut handler = setup_handler_with_sequences(Duration::from_secs(1));

        // Press Ctrl+S (global shortcut)
        let result = handler.process_key_event(
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            EditorMode::Normal,
        );
        assert_eq!(result, MatchResult::Matched(EditorCommand::Save));

        // Now start 'dd' sequence - should work normally
        let result1 = handler.process_key_event(char_key('d'), EditorMode::Normal);
        let result2 = handler.process_key_event(char_key('d'), EditorMode::Normal);
        assert_eq!(result1, MatchResult::Partial);
        assert_eq!(result2, MatchResult::Matched(EditorCommand::DeleteChar));
    }
}
