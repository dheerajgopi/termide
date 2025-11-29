//! Integration tests for EditorState file operations

use std::fs;

use tempfile::TempDir;

use crate::buffer::Position;
use crate::editor::EditorState;

#[test]
fn test_from_file_existing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.txt");
    fs::write(&path, "Hello\nWorld").unwrap();

    let state = EditorState::from_file(&path).unwrap();
    assert_eq!(state.buffer().content(), "Hello\nWorld");
    assert!(!state.buffer().is_dirty());
    assert_eq!(state.buffer().file_path(), Some(&path));
}

#[test]
fn test_from_file_non_existing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("new.txt");

    let state = EditorState::from_file(&path).unwrap();
    assert_eq!(state.buffer().content(), "");
    assert!(!state.buffer().is_dirty());
    assert_eq!(state.buffer().file_path(), Some(&path));
}

#[test]
fn test_save_creates_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("new.txt");

    let mut state = EditorState::from_file(&path).unwrap();
    state.handle_char_insert('H', Position::origin());
    state.handle_char_insert('i', Position::new(0, 1));

    state.save().unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Hi");
    assert!(!state.buffer().is_dirty());
    assert_eq!(state.status_message(), Some("Saved successfully"));
}

#[test]
fn test_save_updates_existing_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("existing.txt");
    fs::write(&path, "Original content").unwrap();

    let state = EditorState::from_file(&path).unwrap();
    assert_eq!(state.buffer().content(), "Original content");

    // Clear and insert new content
    let mut state = EditorState::from_file(&path).unwrap();
    state.buffer_mut().insert_char('N', Position::origin());
    state.buffer_mut().insert_char('e', Position::new(0, 1));
    state.buffer_mut().insert_char('w', Position::new(0, 2));

    state.save().unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.starts_with("New"));
}

#[test]
fn test_full_workflow_create_edit_save() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("workflow.txt");

    // Create new file
    let mut state = EditorState::from_file(&path).unwrap();
    assert!(!path.exists());

    // Edit content
    state.handle_char_insert('L', Position::origin());
    state.handle_char_insert('i', Position::new(0, 1));
    state.handle_char_insert('n', Position::new(0, 2));
    state.handle_char_insert('e', Position::new(0, 3));
    state.handle_char_insert(' ', Position::new(0, 4));
    state.handle_char_insert('1', Position::new(0, 5));
    state.handle_char_insert('\n', Position::new(0, 6));
    state.handle_char_insert('L', Position::new(1, 0));
    state.handle_char_insert('i', Position::new(1, 1));
    state.handle_char_insert('n', Position::new(1, 2));
    state.handle_char_insert('e', Position::new(1, 3));
    state.handle_char_insert(' ', Position::new(1, 4));
    state.handle_char_insert('2', Position::new(1, 5));

    assert!(state.buffer().is_dirty());

    // Save
    state.save().unwrap();
    assert!(path.exists());
    assert!(!state.buffer().is_dirty());

    // Verify content
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Line 1\nLine 2");

    // Reload and verify
    let reloaded = EditorState::from_file(&path).unwrap();
    assert_eq!(reloaded.buffer().content(), "Line 1\nLine 2");
}

#[test]
fn test_save_with_unicode() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("unicode.txt");

    let mut state = EditorState::from_file(&path).unwrap();
    let text = "Hello 世界 🌍";
    for (i, ch) in text.chars().enumerate() {
        state.handle_char_insert(ch, Position::new(0, i));
    }

    state.save().unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, text);
}

#[test]
fn test_load_edit_save_preserves_content() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("preserve.txt");
    let initial = "Line 1\nLine 2\nLine 3";
    fs::write(&path, initial).unwrap();

    let mut state = EditorState::from_file(&path).unwrap();
    assert_eq!(state.buffer().content(), initial);

    // Edit: insert at beginning of line 2
    state.handle_char_insert('X', Position::new(1, 0));

    state.save().unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Line 1\nXLine 2\nLine 3");
}
