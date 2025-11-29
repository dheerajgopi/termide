//! Unit tests for EditorMode

use crate::editor::EditorMode;

#[test]
fn test_editor_mode_default() {
    let mode = EditorMode::default();
    assert_eq!(mode, EditorMode::Insert);
}

#[test]
fn test_editor_mode_to_string() {
    assert_eq!(EditorMode::Insert.to_string(), "INSERT");
    assert_eq!(EditorMode::Normal.to_string(), "NORMAL");
}
