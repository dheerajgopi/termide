//! Integration tests for UI rendering
//!
//! These tests verify rendering behavior with real editor state and buffers.
//! Note: Terminal-dependent rendering cannot be fully tested in automated tests,
//! but we can test state management and rendering logic.

use std::path::PathBuf;
use tempfile::TempDir;
use termide::buffer::Position;
use termide::editor::{EditorMode, EditorState};

#[test]
fn test_render_empty_buffer() {
    let state = EditorState::new();
    let cursor = Position::origin();

    // Should not panic with empty buffer
    assert_eq!(state.buffer().content(), "");
    assert_eq!(state.buffer().line_count(), 1);
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.column, 0);
}

#[test]
fn test_render_single_line_buffer() {
    let mut state = EditorState::new();
    state.handle_char_insert('H', Position::origin());
    state.handle_char_insert('i', Position::new(0, 1));

    assert_eq!(state.buffer().content(), "Hi");
    assert_eq!(state.buffer().line_count(), 1);
}

#[test]
fn test_render_multiline_buffer() {
    let mut state = EditorState::new();
    state.handle_char_insert('L', Position::origin());
    state.handle_char_insert('1', Position::new(0, 1));
    state.handle_char_insert('\n', Position::new(0, 2));
    state.handle_char_insert('L', Position::new(1, 0));
    state.handle_char_insert('2', Position::new(1, 1));

    assert_eq!(state.buffer().content(), "L1\nL2");
    assert_eq!(state.buffer().line_count(), 2);
}

#[test]
fn test_render_with_file_path() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    let mut state = EditorState::new();
    state.set_file_path(&file_path);

    // Verify file path is set
    assert_eq!(state.buffer().file_path(), Some(&PathBuf::from(&file_path)));
}

#[test]
fn test_render_dirty_indicator() {
    let mut state = EditorState::new();

    // Clean buffer
    assert!(!state.buffer().is_dirty());

    // Make dirty
    state.handle_char_insert('X', Position::origin());
    assert!(state.buffer().is_dirty());
}

#[test]
fn test_render_mode_display() {
    let mut state = EditorState::new();

    // Default mode
    assert_eq!(state.mode(), EditorMode::Insert);
    assert_eq!(state.mode().to_string(), "INSERT");

    // Switch mode
    state.set_mode(EditorMode::Normal);
    assert_eq!(state.mode(), EditorMode::Normal);
    assert_eq!(state.mode().to_string(), "NORMAL");
}

#[test]
fn test_render_status_message() {
    let mut state = EditorState::new();

    // No message initially
    assert_eq!(state.status_message(), None);

    // Set message
    state.set_status_message("Test message".to_string());
    assert_eq!(state.status_message(), Some("Test message"));

    // Clear message
    state.clear_status_message();
    assert_eq!(state.status_message(), None);
}

#[test]
fn test_render_warning_message() {
    let mut state = EditorState::new();
    state.handle_char_insert('A', Position::origin());

    // Trigger quit warning
    let should_quit = state.request_quit();
    assert!(!should_quit);
    assert!(state
        .status_message()
        .unwrap()
        .starts_with("Warning:"));
}

#[test]
fn test_render_success_message() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    let mut state = EditorState::new();
    state.set_file_path(&file_path);
    state.handle_char_insert('A', Position::origin());

    // Save should set success message
    state.save().unwrap();
    assert_eq!(state.status_message(), Some("Saved successfully"));
}

#[test]
fn test_render_position_display() {
    let cursor = Position::new(5, 10);

    // Display is 1-indexed
    let display_line = cursor.line + 1;
    let display_column = cursor.column + 1;

    assert_eq!(display_line, 6);
    assert_eq!(display_column, 11);
}

#[test]
fn test_render_large_buffer() {
    let mut state = EditorState::new();

    // Create buffer with 100 lines
    for i in 0..100 {
        let line_num = format!("Line {}", i);
        for (col, ch) in line_num.chars().enumerate() {
            state.handle_char_insert(ch, Position::new(i, col));
        }
        if i < 99 {
            state.handle_char_insert('\n', Position::new(i, line_num.len()));
        }
    }

    assert_eq!(state.buffer().line_count(), 100);

    // Verify first and last line
    assert!(state.buffer().get_line(0).unwrap().starts_with("Line 0"));
    assert!(state.buffer().get_line(99).unwrap().starts_with("Line 99"));
}

#[test]
fn test_render_unicode_content() {
    let mut state = EditorState::new();

    // Insert unicode characters
    state.handle_char_insert('你', Position::origin());
    state.handle_char_insert('好', Position::new(0, 1));
    state.handle_char_insert('😊', Position::new(0, 2));

    assert_eq!(state.buffer().content(), "你好😊");
    assert_eq!(state.buffer().len_chars(), 3);
}

#[test]
fn test_render_special_characters() {
    let mut state = EditorState::new();

    // Tab character
    state.handle_char_insert('\t', Position::origin());
    state.handle_char_insert('A', Position::new(0, 1));

    assert_eq!(state.buffer().content(), "\tA");

    // Newline
    state.handle_char_insert('\n', Position::new(0, 2));
    state.handle_char_insert('B', Position::new(1, 0));

    assert_eq!(state.buffer().content(), "\tA\nB");
}

#[test]
fn test_viewport_calculation_logic() {
    // Simulate viewport for 100-line buffer on 24-line terminal
    let _buffer_lines = 100;  // Documented in comment, used for test context
    let terminal_height = 24;
    let status_bar_height = 2;
    let visible_height = terminal_height - status_bar_height;

    // Cursor at line 0
    let cursor_line = 0;
    let scroll_offset = 0;
    assert!(cursor_line >= scroll_offset);
    assert!(cursor_line < scroll_offset + visible_height);

    // Cursor at line 50 (middle of file)
    let cursor_line = 50;
    let mut scroll_offset = 0;

    // Should scroll down to show cursor
    if cursor_line >= scroll_offset + visible_height {
        scroll_offset = cursor_line - visible_height + 1;
    }
    assert_eq!(scroll_offset, 29); // 50 - 22 + 1

    // Cursor at line 99 (end of file)
    let cursor_line = 99;

    if cursor_line >= scroll_offset + visible_height {
        scroll_offset = cursor_line - visible_height + 1;
    }
    assert_eq!(scroll_offset, 78); // 99 - 22 + 1
}

#[test]
fn test_no_filename_display() {
    let state = EditorState::new();

    // No file path set
    let filename = state
        .buffer()
        .file_path()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("[No Name]");

    assert_eq!(filename, "[No Name]");
}

#[test]
fn test_filename_display() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("example.txt");

    let mut state = EditorState::new();
    state.set_file_path(&file_path);

    let filename = state
        .buffer()
        .file_path()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("[No Name]");

    assert_eq!(filename, "example.txt");
}

#[test]
fn test_render_after_delete() {
    let mut state = EditorState::new();

    state.handle_char_insert('A', Position::origin());
    state.handle_char_insert('B', Position::new(0, 1));
    state.handle_char_insert('C', Position::new(0, 2));

    assert_eq!(state.buffer().content(), "ABC");

    // Delete middle character
    state.handle_char_delete(Position::new(0, 1));

    assert_eq!(state.buffer().content(), "AC");
}

#[test]
fn test_render_empty_lines() {
    let mut state = EditorState::new();

    // Create buffer with empty lines
    state.handle_char_insert('\n', Position::origin());
    state.handle_char_insert('\n', Position::new(1, 0));
    state.handle_char_insert('A', Position::new(2, 0));

    assert_eq!(state.buffer().content(), "\n\nA");
    assert_eq!(state.buffer().line_count(), 3);
}
