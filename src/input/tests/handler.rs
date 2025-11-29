//! Unit tests for key event handler

use crate::input::{handle_key_event, EditorCommand, Direction};
use crate::editor::EditorMode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// ============================================================================
// Insert Mode Tests
// ============================================================================

#[test]
fn test_insert_mode_printable_characters() {
    let mode = EditorMode::Insert;

    // Regular characters
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE), mode),
        Some(EditorCommand::InsertChar('a'))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::NONE), mode),
        Some(EditorCommand::InsertChar('Z'))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE), mode),
        Some(EditorCommand::InsertChar('5'))
    );

    // Special characters
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE), mode),
        Some(EditorCommand::InsertChar('!'))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE), mode),
        Some(EditorCommand::InsertChar(' '))
    );
}

#[test]
fn test_insert_mode_unicode_characters() {
    let mode = EditorMode::Insert;

    // Unicode characters should be inserted
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('ñ'), KeyModifiers::NONE), mode),
        Some(EditorCommand::InsertChar('ñ'))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('中'), KeyModifiers::NONE), mode),
        Some(EditorCommand::InsertChar('中'))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('🚀'), KeyModifiers::NONE), mode),
        Some(EditorCommand::InsertChar('🚀'))
    );
}

#[test]
fn test_insert_mode_enter_key() {
    let mode = EditorMode::Insert;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), mode),
        Some(EditorCommand::InsertChar('\n'))
    );
}

#[test]
fn test_insert_mode_backspace() {
    let mode = EditorMode::Insert;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE), mode),
        Some(EditorCommand::DeleteChar)
    );
}

#[test]
fn test_insert_mode_escape_to_normal() {
    let mode = EditorMode::Insert;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), mode),
        Some(EditorCommand::ChangeMode(EditorMode::Normal))
    );
}

#[test]
fn test_insert_mode_arrow_keys() {
    let mode = EditorMode::Insert;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), mode),
        Some(EditorCommand::MoveCursor(Direction::Up))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), mode),
        Some(EditorCommand::MoveCursor(Direction::Down))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE), mode),
        Some(EditorCommand::MoveCursor(Direction::Left))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE), mode),
        Some(EditorCommand::MoveCursor(Direction::Right))
    );
}

#[test]
fn test_insert_mode_unmapped_keys() {
    let mode = EditorMode::Insert;

    // Function keys
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE), mode),
        None
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::F(12), KeyModifiers::NONE), mode),
        None
    );

    // Tab (unmapped in base editor)
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE), mode),
        None
    );

    // Delete key
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE), mode),
        None
    );

    // Home/End
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE), mode),
        None
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::End, KeyModifiers::NONE), mode),
        None
    );
}

// ============================================================================
// Normal Mode Tests
// ============================================================================

#[test]
fn test_normal_mode_i_to_insert() {
    let mode = EditorMode::Normal;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE), mode),
        Some(EditorCommand::ChangeMode(EditorMode::Insert))
    );
}

#[test]
fn test_normal_mode_arrow_navigation() {
    let mode = EditorMode::Normal;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), mode),
        Some(EditorCommand::MoveCursor(Direction::Up))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), mode),
        Some(EditorCommand::MoveCursor(Direction::Down))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE), mode),
        Some(EditorCommand::MoveCursor(Direction::Left))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE), mode),
        Some(EditorCommand::MoveCursor(Direction::Right))
    );
}

#[test]
fn test_normal_mode_regular_chars_ignored() {
    let mode = EditorMode::Normal;

    // Regular characters (except 'i') should be ignored in normal mode
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE), mode),
        None
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE), mode),
        None
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE), mode),
        None
    );
}

#[test]
fn test_normal_mode_unmapped_keys() {
    let mode = EditorMode::Normal;

    // Function keys
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE), mode),
        None
    );

    // Enter key
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), mode),
        None
    );

    // Backspace
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE), mode),
        None
    );
}

// ============================================================================
// Special Key Combinations (Both Modes)
// ============================================================================

#[test]
fn test_ctrl_s_saves_in_insert_mode() {
    let mode = EditorMode::Insert;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL), mode),
        Some(EditorCommand::Save)
    );

    // Case-insensitive
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('S'), KeyModifiers::CONTROL), mode),
        Some(EditorCommand::Save)
    );
}

#[test]
fn test_ctrl_s_saves_in_normal_mode() {
    let mode = EditorMode::Normal;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL), mode),
        Some(EditorCommand::Save)
    );

    // Case-insensitive
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('S'), KeyModifiers::CONTROL), mode),
        Some(EditorCommand::Save)
    );
}

#[test]
fn test_ctrl_q_quits_in_insert_mode() {
    let mode = EditorMode::Insert;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL), mode),
        Some(EditorCommand::Quit)
    );

    // Case-insensitive
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::CONTROL), mode),
        Some(EditorCommand::Quit)
    );
}

#[test]
fn test_ctrl_q_quits_in_normal_mode() {
    let mode = EditorMode::Normal;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL), mode),
        Some(EditorCommand::Quit)
    );

    // Case-insensitive
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::CONTROL), mode),
        Some(EditorCommand::Quit)
    );
}

#[test]
fn test_other_ctrl_combinations_ignored() {
    // Test that other Ctrl combinations don't trigger special behavior
    let mode = EditorMode::Insert;

    // Ctrl+A through Ctrl+Z (except Ctrl+S and Ctrl+Q)
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL), mode),
        None
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL), mode),
        None
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL), mode),
        None
    );
}

// ============================================================================
// Mode-Specific Behavior Tests
// ============================================================================

#[test]
fn test_char_i_behaves_differently_per_mode() {
    // In Insert mode, 'i' should insert the character
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE), EditorMode::Insert),
        Some(EditorCommand::InsertChar('i'))
    );

    // In Normal mode, 'i' should switch to Insert mode
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE), EditorMode::Normal),
        Some(EditorCommand::ChangeMode(EditorMode::Insert))
    );
}

#[test]
fn test_ctrl_i_does_not_switch_mode() {
    // Ctrl+I should not switch to insert mode (only plain 'i')
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::CONTROL), EditorMode::Normal),
        None
    );
}

#[test]
fn test_escape_only_works_in_insert_mode() {
    // Escape in Insert mode switches to Normal
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), EditorMode::Insert),
        Some(EditorCommand::ChangeMode(EditorMode::Normal))
    );

    // Escape in Normal mode does nothing
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), EditorMode::Normal),
        None
    );
}

#[test]
fn test_backspace_only_works_in_insert_mode() {
    // Backspace in Insert mode deletes character
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE), EditorMode::Insert),
        Some(EditorCommand::DeleteChar)
    );

    // Backspace in Normal mode does nothing
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE), EditorMode::Normal),
        None
    );
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_shift_modified_characters_treated_as_regular() {
    let mode = EditorMode::Insert;

    // Shift+A results in 'A' character, which should be inserted
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT), mode),
        Some(EditorCommand::InsertChar('A'))
    );

    // Shift+1 results in '!' on most keyboards
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('!'), KeyModifiers::SHIFT), mode),
        Some(EditorCommand::InsertChar('!'))
    );
}

#[test]
fn test_alt_modified_keys_ignored() {
    let mode = EditorMode::Insert;

    // Alt combinations are not mapped
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT), mode),
        None
    );
}

#[test]
fn test_multiple_modifiers_ctrl_takes_precedence() {
    let mode = EditorMode::Insert;

    // Ctrl+Shift+S should still save
    let modifiers = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('S'), modifiers), mode),
        Some(EditorCommand::Save)
    );
}

// ============================================================================
// Prompt Mode Tests
// ============================================================================

#[test]
fn test_prompt_mode_printable_characters() {
    let mode = EditorMode::Prompt;

    // Regular characters should insert into prompt
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE), mode),
        Some(EditorCommand::PromptInsertChar('a'))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::NONE), mode),
        Some(EditorCommand::PromptInsertChar('Z'))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE), mode),
        Some(EditorCommand::PromptInsertChar('5'))
    );

    // Special characters
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('.'), KeyModifiers::NONE), mode),
        Some(EditorCommand::PromptInsertChar('.'))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE), mode),
        Some(EditorCommand::PromptInsertChar('/'))
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('_'), KeyModifiers::NONE), mode),
        Some(EditorCommand::PromptInsertChar('_'))
    );
}

#[test]
fn test_prompt_mode_backspace() {
    let mode = EditorMode::Prompt;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE), mode),
        Some(EditorCommand::PromptDeleteChar)
    );
}

#[test]
fn test_prompt_mode_enter_accepts() {
    let mode = EditorMode::Prompt;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), mode),
        Some(EditorCommand::AcceptPrompt)
    );
}

#[test]
fn test_prompt_mode_escape_cancels() {
    let mode = EditorMode::Prompt;

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), mode),
        Some(EditorCommand::CancelPrompt)
    );
}

#[test]
fn test_prompt_mode_ignores_ctrl_s_and_ctrl_q() {
    let mode = EditorMode::Prompt;

    // In prompt mode, Ctrl+S and Ctrl+Q should still work
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL), mode),
        Some(EditorCommand::Save)
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL), mode),
        Some(EditorCommand::Quit)
    );
}

#[test]
fn test_prompt_mode_arrow_keys_ignored() {
    let mode = EditorMode::Prompt;

    // Arrow keys don't do anything in prompt mode
    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), mode),
        None
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), mode),
        None
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE), mode),
        None
    );

    assert_eq!(
        handle_key_event(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE), mode),
        None
    );
}
