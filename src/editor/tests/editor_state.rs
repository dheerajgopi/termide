//! Unit tests for EditorState

use std::path::PathBuf;

use crate::buffer::Position;
use crate::editor::{EditorMode, EditorState};

#[test]
fn test_editor_state_new() {
    let state = EditorState::new();
    assert_eq!(state.buffer().content(), "");
    assert!(!state.buffer().is_dirty());
    assert_eq!(state.mode(), EditorMode::Insert);
    assert_eq!(state.status_message(), None);
    assert!(!state.should_quit());
}

#[test]
fn test_editor_state_default() {
    let state = EditorState::default();
    assert_eq!(state.mode(), EditorMode::Insert);
    assert!(!state.should_quit());
}

#[test]
fn test_mode_switching() {
    let mut state = EditorState::new();
    assert_eq!(state.mode(), EditorMode::Insert);

    state.set_mode(EditorMode::Normal);
    assert_eq!(state.mode(), EditorMode::Normal);

    state.set_mode(EditorMode::Insert);
    assert_eq!(state.mode(), EditorMode::Insert);
}

#[test]
fn test_status_message_management() {
    let mut state = EditorState::new();
    assert_eq!(state.status_message(), None);

    state.set_status_message("Test message".to_string());
    assert_eq!(state.status_message(), Some("Test message"));

    state.clear_status_message();
    assert_eq!(state.status_message(), None);
}

#[test]
fn test_handle_char_insert() {
    let mut state = EditorState::new();
    assert!(state.handle_char_insert('H', Position::origin()));
    assert!(state.handle_char_insert('i', Position::new(0, 1)));
    assert_eq!(state.buffer().content(), "Hi");
    assert!(state.buffer().is_dirty());
}

#[test]
fn test_handle_char_delete() {
    let mut state = EditorState::new();
    state.handle_char_insert('A', Position::origin());
    state.handle_char_insert('B', Position::new(0, 1));
    state.handle_char_insert('C', Position::new(0, 2));

    assert!(state.handle_char_delete(Position::new(0, 1)));
    assert_eq!(state.buffer().content(), "AC");
}

#[test]
fn test_request_quit_clean_buffer() {
    let mut state = EditorState::new();
    assert!(!state.buffer().is_dirty());

    assert!(state.request_quit());
    assert!(state.should_quit());
    assert_eq!(state.status_message(), None);
}

#[test]
fn test_request_quit_dirty_buffer_requires_confirmation() {
    let mut state = EditorState::new();
    state.handle_char_insert('A', Position::origin());
    assert!(state.buffer().is_dirty());

    // First attempt - should warn
    assert!(!state.request_quit());
    assert!(!state.should_quit());
    assert_eq!(
        state.status_message(),
        Some("Warning: Unsaved changes! Press Ctrl+Q again to force quit.")
    );

    // Second attempt - should quit
    assert!(state.request_quit());
    assert!(state.should_quit());
}

#[test]
fn test_force_quit() {
    let mut state = EditorState::new();
    state.handle_char_insert('A', Position::origin());
    assert!(state.buffer().is_dirty());

    state.force_quit();
    assert!(state.should_quit());
}

#[test]
fn test_set_file_path() {
    let mut state = EditorState::new();
    let path = PathBuf::from("test.txt");

    state.set_file_path(&path);
    assert_eq!(state.buffer().file_path(), Some(&path));
}

#[test]
fn test_save_without_file_path_enters_prompt() {
    let mut state = EditorState::new();
    state.handle_char_insert('A', Position::origin());

    let result = state.save();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false); // Did not save, entered prompt mode instead
    assert_eq!(state.mode(), EditorMode::Prompt);
    assert_eq!(state.prompt_message(), "Save as: ");
}

#[test]
fn test_buffer_access() {
    let mut state = EditorState::new();

    // Test immutable access
    assert_eq!(state.buffer().content(), "");

    // Test mutable access
    state.buffer_mut().insert_char('X', Position::origin());
    assert_eq!(state.buffer().content(), "X");
}

#[test]
fn test_enter_prompt() {
    let mut state = EditorState::new();
    assert_eq!(state.mode(), EditorMode::Insert);

    state.enter_prompt("Enter filename: ".to_string());
    assert_eq!(state.mode(), EditorMode::Prompt);
    assert_eq!(state.prompt_message(), "Enter filename: ");
    assert_eq!(state.prompt_input(), "");
}

#[test]
fn test_prompt_insert_char() {
    let mut state = EditorState::new();
    state.enter_prompt("Save as: ".to_string());

    state.prompt_insert_char('t');
    assert_eq!(state.prompt_input(), "t");

    state.prompt_insert_char('e');
    state.prompt_insert_char('s');
    state.prompt_insert_char('t');
    assert_eq!(state.prompt_input(), "test");
}

#[test]
fn test_prompt_delete_char() {
    let mut state = EditorState::new();
    state.enter_prompt("Save as: ".to_string());

    state.prompt_insert_char('t');
    state.prompt_insert_char('e');
    state.prompt_insert_char('s');
    state.prompt_insert_char('t');
    assert_eq!(state.prompt_input(), "test");

    state.prompt_delete_char();
    assert_eq!(state.prompt_input(), "tes");

    state.prompt_delete_char();
    state.prompt_delete_char();
    assert_eq!(state.prompt_input(), "t");

    state.prompt_delete_char();
    assert_eq!(state.prompt_input(), "");
}

#[test]
fn test_accept_prompt() {
    let mut state = EditorState::new();
    state.set_mode(EditorMode::Normal);
    state.enter_prompt("Save as: ".to_string());

    state.prompt_insert_char('t');
    state.prompt_insert_char('e');
    state.prompt_insert_char('s');
    state.prompt_insert_char('t');

    let result = state.accept_prompt();
    assert_eq!(result, "test");
    assert_eq!(state.mode(), EditorMode::Normal); // Returns to previous mode
    assert_eq!(state.prompt_input(), "");
    assert_eq!(state.prompt_message(), "");
}

#[test]
fn test_cancel_prompt() {
    let mut state = EditorState::new();
    state.set_mode(EditorMode::Insert);
    state.enter_prompt("Save as: ".to_string());

    state.prompt_insert_char('t');
    state.prompt_insert_char('e');
    state.prompt_insert_char('s');
    state.prompt_insert_char('t');

    state.cancel_prompt();
    assert_eq!(state.mode(), EditorMode::Insert); // Returns to previous mode
    assert_eq!(state.prompt_input(), "");
    assert_eq!(state.prompt_message(), "");
}
