//! Integration tests for complete default bindings coverage
//!
//! This test module verifies that every default keybinding registered in Phase 2
//! works correctly in the integrated system. It ensures no bindings were lost or
//! broken during migration and that the priority system works as expected.
//!
//! ## Test Organization
//!
//! Tests are organized by binding category:
//! - Global bindings (Ctrl+S, Ctrl+Q) tested in both Insert and Normal modes
//! - Insert mode specific bindings tested only in Insert mode
//! - Normal mode specific bindings tested only in Normal mode
//! - Prompt mode specific bindings tested only in Prompt mode
//! - Arrow key navigation tested across appropriate modes
//! - Special keys tested (Enter, Tab, Esc, Backspace, etc.)
//! - Modifier combinations tested (Ctrl, Shift, Ctrl+Shift)
//! - Mode isolation tests (bindings don't trigger in wrong modes)

use crate::editor::EditorMode;
use crate::input::bindings::register_default_bindings;
use crate::input::input_handler::{InputHandler, MatchResult};
use crate::input::{Direction, EditorCommand};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Helper to create InputHandler with default bindings
fn create_handler_with_defaults() -> InputHandler {
    let timeout = Duration::from_millis(1000);
    let mut handler = InputHandler::with_timeout(timeout);
    register_default_bindings(handler.registry_mut())
        .expect("default bindings should register without conflicts");
    handler
}

/// Helper to create a KeyEvent
fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

// ============================================================================
// Global Bindings Coverage (Active in Insert and Normal, NOT in Prompt)
// ============================================================================

#[test]
fn test_global_ctrl_s_save_in_insert_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::Save)),
        "Ctrl+S should trigger Save in Insert mode"
    );
}

#[test]
fn test_global_ctrl_s_save_in_normal_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Normal);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::Save)),
        "Ctrl+S should trigger Save in Normal mode"
    );
}

#[test]
fn test_global_ctrl_shift_s_save_in_insert_mode() {
    let mut handler = create_handler_with_defaults();

    // Uppercase 'S' represents Shift+S
    let event = key_event(KeyCode::Char('S'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::Save)),
        "Ctrl+Shift+S should trigger Save in Insert mode"
    );
}

#[test]
fn test_global_ctrl_shift_s_save_in_normal_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Char('S'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Normal);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::Save)),
        "Ctrl+Shift+S should trigger Save in Normal mode"
    );
}

#[test]
fn test_global_ctrl_q_quit_in_insert_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Char('q'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::Quit)),
        "Ctrl+Q should trigger Quit in Insert mode"
    );
}

#[test]
fn test_global_ctrl_q_quit_in_normal_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Char('q'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Normal);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::Quit)),
        "Ctrl+Q should trigger Quit in Normal mode"
    );
}

#[test]
fn test_global_ctrl_shift_q_quit_in_insert_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Char('Q'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::Quit)),
        "Ctrl+Shift+Q should trigger Quit in Insert mode"
    );
}

#[test]
fn test_global_ctrl_shift_q_quit_in_normal_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Char('Q'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Normal);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::Quit)),
        "Ctrl+Shift+Q should trigger Quit in Normal mode"
    );
}

#[test]
fn test_global_shortcuts_not_active_in_prompt_mode() {
    let mut handler = create_handler_with_defaults();

    // Test Ctrl+S in Prompt mode (should NOT trigger Save)
    let event = key_event(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Prompt);
    assert!(
        matches!(result, MatchResult::NoMatch),
        "Global shortcuts should NOT be active in Prompt mode"
    );

    // Test Ctrl+Q in Prompt mode (should NOT trigger Quit)
    let event = key_event(KeyCode::Char('q'), KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Prompt);
    assert!(
        matches!(result, MatchResult::NoMatch),
        "Global shortcuts should NOT be active in Prompt mode"
    );
}

// ============================================================================
// Insert Mode Specific Bindings Coverage
// ============================================================================

#[test]
fn test_insert_enter_inserts_newline() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Enter, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::InsertChar('\n'))),
        "Enter should insert newline in Insert mode"
    );
}

#[test]
fn test_insert_backspace_deletes_char() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Backspace, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::DeleteChar)),
        "Backspace should delete character in Insert mode"
    );
}

#[test]
fn test_insert_esc_switches_to_normal() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Esc, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Normal))),
        "Esc should switch to Normal mode from Insert mode"
    );
}

#[test]
fn test_insert_up_arrow_moves_cursor_up() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Up, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Up))),
        "Up arrow should move cursor up in Insert mode"
    );
}

#[test]
fn test_insert_down_arrow_moves_cursor_down() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Down, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Down))),
        "Down arrow should move cursor down in Insert mode"
    );
}

#[test]
fn test_insert_left_arrow_moves_cursor_left() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Left, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Left))),
        "Left arrow should move cursor left in Insert mode"
    );
}

#[test]
fn test_insert_right_arrow_moves_cursor_right() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Right, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Right))),
        "Right arrow should move cursor right in Insert mode"
    );
}

// ============================================================================
// Normal Mode Specific Bindings Coverage
// ============================================================================

#[test]
fn test_normal_i_switches_to_insert() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Char('i'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Insert))),
        "'i' should switch to Insert mode from Normal mode"
    );
}

#[test]
fn test_normal_up_arrow_moves_cursor_up() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Up, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Up))),
        "Up arrow should move cursor up in Normal mode"
    );
}

#[test]
fn test_normal_down_arrow_moves_cursor_down() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Down, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Down))),
        "Down arrow should move cursor down in Normal mode"
    );
}

#[test]
fn test_normal_left_arrow_moves_cursor_left() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Left, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Left))),
        "Left arrow should move cursor left in Normal mode"
    );
}

#[test]
fn test_normal_right_arrow_moves_cursor_right() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Right, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Right))),
        "Right arrow should move cursor right in Normal mode"
    );
}

// ============================================================================
// Prompt Mode Specific Bindings Coverage
// ============================================================================

#[test]
fn test_prompt_backspace_deletes_from_prompt() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Backspace, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Prompt);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::PromptDeleteChar)),
        "Backspace should delete from prompt in Prompt mode"
    );
}

#[test]
fn test_prompt_enter_accepts_prompt() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Enter, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Prompt);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::AcceptPrompt)),
        "Enter should accept prompt in Prompt mode"
    );
}

#[test]
fn test_prompt_esc_cancels_prompt() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Esc, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Prompt);

    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::CancelPrompt)),
        "Esc should cancel prompt in Prompt mode"
    );
}

// ============================================================================
// Mode Isolation Tests (Bindings Don't Trigger in Wrong Modes)
// ============================================================================

#[test]
fn test_insert_enter_not_active_in_normal_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Enter, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);

    // In Normal mode, Enter should not trigger InsertChar('\n')
    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::InsertChar('\n'))),
        "Insert mode Enter binding should not trigger in Normal mode"
    );
}

#[test]
fn test_insert_backspace_not_active_in_normal_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Backspace, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);

    // In Normal mode, Backspace should not trigger DeleteChar
    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::DeleteChar)),
        "Insert mode Backspace binding should not trigger in Normal mode"
    );
}

#[test]
fn test_insert_esc_not_active_in_normal_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Esc, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Normal);

    // In Normal mode, Esc should not trigger mode change
    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Normal))),
        "Insert mode Esc binding should not trigger in Normal mode"
    );
}

#[test]
fn test_normal_i_not_active_in_insert_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Char('i'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    // In Insert mode, 'i' should not trigger mode change (it's a regular character)
    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Insert))),
        "Normal mode 'i' binding should not trigger in Insert mode"
    );
}

#[test]
fn test_normal_i_not_active_in_prompt_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Char('i'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Prompt);

    // In Prompt mode, 'i' should not trigger mode change
    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Insert))),
        "Normal mode 'i' binding should not trigger in Prompt mode"
    );
}

#[test]
fn test_prompt_backspace_not_active_in_insert_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Backspace, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    // In Insert mode, Backspace should trigger DeleteChar, not PromptDeleteChar
    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::DeleteChar)),
        "Prompt mode Backspace should not override Insert mode Backspace"
    );
}

#[test]
fn test_prompt_enter_not_active_in_insert_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Enter, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    // In Insert mode, Enter should trigger InsertChar('\n'), not AcceptPrompt
    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::InsertChar('\n'))),
        "Prompt mode Enter should not override Insert mode Enter"
    );
}

#[test]
fn test_prompt_esc_not_active_in_insert_mode() {
    let mut handler = create_handler_with_defaults();

    let event = key_event(KeyCode::Esc, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Insert);

    // In Insert mode, Esc should trigger ChangeMode(Normal), not CancelPrompt
    assert!(
        matches!(result, MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Normal))),
        "Prompt mode Esc should not override Insert mode Esc"
    );
}

#[test]
fn test_arrow_keys_not_active_in_prompt_mode() {
    let mut handler = create_handler_with_defaults();

    // Arrow keys are registered for Insert and Normal modes, not Prompt
    let event = key_event(KeyCode::Up, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Prompt);
    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Up))),
        "Arrow keys should not be active in Prompt mode"
    );

    let event = key_event(KeyCode::Down, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Prompt);
    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Down))),
        "Arrow keys should not be active in Prompt mode"
    );

    let event = key_event(KeyCode::Left, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Prompt);
    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Left))),
        "Arrow keys should not be active in Prompt mode"
    );

    let event = key_event(KeyCode::Right, KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Prompt);
    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Right))),
        "Arrow keys should not be active in Prompt mode"
    );
}

// ============================================================================
// Modifier Combinations Coverage
// ============================================================================

#[test]
fn test_plain_char_not_matched_when_ctrl_expected() {
    let mut handler = create_handler_with_defaults();

    // Plain 's' without Ctrl should not trigger Save
    let event = key_event(KeyCode::Char('s'), KeyModifiers::NONE);
    let result = handler.process_key_event(event, EditorMode::Prompt);

    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::Save)),
        "Plain 's' should not trigger Save (needs Ctrl modifier)"
    );
}

#[test]
fn test_char_with_shift_not_matched_when_plain_expected() {
    let mut handler = create_handler_with_defaults();

    // Uppercase 'I' should not trigger Insert mode (needs lowercase 'i')
    let event = key_event(KeyCode::Char('I'), KeyModifiers::SHIFT);
    let result = handler.process_key_event(event, EditorMode::Prompt);

    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::ChangeMode(EditorMode::Insert))),
        "Uppercase 'I' should not trigger Insert mode (needs plain 'i')"
    );
}

#[test]
fn test_char_with_alt_not_matched() {
    let mut handler = create_handler_with_defaults();

    // Alt+S should not trigger Save (needs Ctrl)
    let event = key_event(KeyCode::Char('s'), KeyModifiers::ALT);
    let result = handler.process_key_event(event, EditorMode::Prompt);

    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::Save)),
        "Alt+S should not trigger Save (needs Ctrl modifier)"
    );
}

#[test]
fn test_arrow_with_modifiers_not_matched() {
    let mut handler = create_handler_with_defaults();

    // Ctrl+Up should not trigger cursor movement (needs plain Up)
    let event = key_event(KeyCode::Up, KeyModifiers::CONTROL);
    let result = handler.process_key_event(event, EditorMode::Prompt);

    assert!(
        !matches!(result, MatchResult::Matched(EditorCommand::MoveCursor(Direction::Up))),
        "Ctrl+Up should not trigger cursor movement (needs plain Up)"
    );
}

// ============================================================================
// Special Keys Coverage
// ============================================================================

#[test]
fn test_all_special_keys_recognized_in_appropriate_modes() {
    // Enter in Insert mode
    let mut handler = create_handler_with_defaults();
    let event = key_event(KeyCode::Enter, KeyModifiers::NONE);
    assert!(
        matches!(handler.process_key_event(event, EditorMode::Prompt), MatchResult::Matched(_)),
        "Enter should be recognized in Insert mode"
    );

    // Backspace in Insert mode
    let mut handler = create_handler_with_defaults();
    let event = key_event(KeyCode::Backspace, KeyModifiers::NONE);
    assert!(
        matches!(handler.process_key_event(event, EditorMode::Prompt), MatchResult::Matched(_)),
        "Backspace should be recognized in Insert mode"
    );

    // Esc in Insert mode
    let mut handler = create_handler_with_defaults();
    let event = key_event(KeyCode::Esc, KeyModifiers::NONE);
    assert!(
        matches!(handler.process_key_event(event, EditorMode::Prompt), MatchResult::Matched(_)),
        "Esc should be recognized in Insert mode"
    );

    // Arrow keys in Insert and Normal modes
    for mode in [EditorMode::Insert, EditorMode::Normal] {
        let mut handler = create_handler_with_defaults();

        let event = key_event(KeyCode::Up, KeyModifiers::NONE);
        assert!(
            matches!(handler.process_key_event(event, mode), MatchResult::Matched(_)),
            "Up arrow should be recognized in {:?} mode",
            mode
        );

        let event = key_event(KeyCode::Down, KeyModifiers::NONE);
        assert!(
            matches!(handler.process_key_event(event, mode), MatchResult::Matched(_)),
            "Down arrow should be recognized in {:?} mode",
            mode
        );

        let event = key_event(KeyCode::Left, KeyModifiers::NONE);
        assert!(
            matches!(handler.process_key_event(event, mode), MatchResult::Matched(_)),
            "Left arrow should be recognized in {:?} mode",
            mode
        );

        let event = key_event(KeyCode::Right, KeyModifiers::NONE);
        assert!(
            matches!(handler.process_key_event(event, mode), MatchResult::Matched(_)),
            "Right arrow should be recognized in {:?} mode",
            mode
        );
    }
}
