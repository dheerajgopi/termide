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
fn test_save_without_file_path() {
    let mut state = EditorState::new();
    state.handle_char_insert('A', Position::origin());

    let result = state.save();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No file path associated with buffer"));
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
