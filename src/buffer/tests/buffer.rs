//! Unit tests for Buffer struct

use crate::buffer::{Buffer, Position};
use std::path::PathBuf;

#[test]
fn test_buffer_new() {
    let buffer = Buffer::new();
    assert_eq!(buffer.content(), "");
    assert_eq!(buffer.line_count(), 1); // Empty buffer has 1 line
    assert!(!buffer.is_dirty());
    assert!(buffer.is_empty());
}

#[test]
fn test_buffer_from_str() {
    let buffer = Buffer::from_str("Hello\nWorld");
    assert_eq!(buffer.content(), "Hello\nWorld");
    assert_eq!(buffer.line_count(), 2);
    assert!(!buffer.is_dirty());
}

#[test]
fn test_insert_char() {
    let mut buffer = Buffer::new();

    // Insert at origin
    assert!(buffer.insert_char('H', Position::origin()));
    assert_eq!(buffer.content(), "H");
    assert!(buffer.is_dirty());

    // Insert at end
    assert!(buffer.insert_char('i', Position { line: 0, column: 1 }));
    assert_eq!(buffer.content(), "Hi");

    // Insert in middle
    assert!(buffer.insert_char('e', Position { line: 0, column: 1 }));
    assert_eq!(buffer.content(), "Hei");
}

#[test]
fn test_insert_newline() {
    let mut buffer = Buffer::new();

    buffer.insert_char('A', Position::origin());
    buffer.insert_char('\n', Position { line: 0, column: 1 });
    buffer.insert_char('B', Position { line: 1, column: 0 });

    assert_eq!(buffer.content(), "A\nB");
    assert_eq!(buffer.line_count(), 2);
}

#[test]
fn test_delete_char() {
    let mut buffer = Buffer::from_str("ABC");

    // Delete middle character
    assert!(buffer.delete_char_at(Position { line: 0, column: 1 }));
    assert_eq!(buffer.content(), "AC");
    assert!(buffer.is_dirty());

    // Delete first character
    assert!(buffer.delete_char_at(Position { line: 0, column: 0 }));
    assert_eq!(buffer.content(), "C");
}

#[test]
fn test_delete_newline() {
    let mut buffer = Buffer::from_str("A\nB");

    // Delete newline character
    assert!(buffer.delete_char_at(Position { line: 0, column: 1 }));
    assert_eq!(buffer.content(), "AB");
    assert_eq!(buffer.line_count(), 1);
}

#[test]
fn test_line_count() {
    let buffer = Buffer::from_str("Line 1\nLine 2\nLine 3");
    assert_eq!(buffer.line_count(), 3);

    let empty = Buffer::new();
    assert_eq!(empty.line_count(), 1);
}

#[test]
fn test_get_line() {
    let buffer = Buffer::from_str("First\nSecond\nThird");

    assert_eq!(buffer.get_line(0), Some("First\n".to_string()));
    assert_eq!(buffer.get_line(1), Some("Second\n".to_string()));
    assert_eq!(buffer.get_line(2), Some("Third".to_string())); // Last line has no newline
    assert_eq!(buffer.get_line(3), None);
}

#[test]
fn test_line_len() {
    let buffer = Buffer::from_str("Hello\nWorld!");

    assert_eq!(buffer.line_len(0), Some(5)); // "Hello" (without \n)
    assert_eq!(buffer.line_len(1), Some(6)); // "World!"
    assert_eq!(buffer.line_len(2), None);
}

#[test]
fn test_dirty_flag() {
    let mut buffer = Buffer::new();
    assert!(!buffer.is_dirty());

    buffer.insert_char('X', Position::origin());
    assert!(buffer.is_dirty());

    buffer.clear_dirty();
    assert!(!buffer.is_dirty());

    buffer.delete_char_at(Position::origin());
    assert!(buffer.is_dirty());
}

#[test]
fn test_file_path() {
    let mut buffer = Buffer::new();
    assert_eq!(buffer.file_path(), None);

    let path = PathBuf::from("/tmp/test.txt");
    buffer.set_file_path(path.clone());
    assert_eq!(buffer.file_path(), Some(&path));
}

#[test]
fn test_is_valid_position() {
    let buffer = Buffer::from_str("Hello\nWorld");

    // Valid positions
    assert!(buffer.is_valid_position(Position { line: 0, column: 0 }));
    assert!(buffer.is_valid_position(Position { line: 0, column: 5 })); // At end of line
    assert!(buffer.is_valid_position(Position { line: 1, column: 0 }));
    assert!(buffer.is_valid_position(Position { line: 1, column: 5 })); // At end of last line

    // Invalid positions
    assert!(!buffer.is_valid_position(Position { line: 0, column: 6 })); // Beyond line end
    assert!(!buffer.is_valid_position(Position { line: 2, column: 0 })); // Invalid line
    assert!(!buffer.is_valid_position(Position { line: 10, column: 0 }));
}

#[test]
fn test_clamp_position() {
    let buffer = Buffer::from_str("Hello\nWorld");

    // Already valid
    assert_eq!(
        buffer.clamp_position(Position { line: 0, column: 3 }),
        Position { line: 0, column: 3 }
    );

    // Column beyond line end
    assert_eq!(
        buffer.clamp_position(Position { line: 0, column: 10 }),
        Position { line: 0, column: 5 }
    );

    // Line beyond buffer end
    assert_eq!(
        buffer.clamp_position(Position { line: 5, column: 0 }),
        Position { line: 1, column: 0 }
    );

    // Both beyond bounds
    assert_eq!(
        buffer.clamp_position(Position { line: 10, column: 20 }),
        Position { line: 1, column: 5 }
    );
}

#[test]
fn test_insert_boundary_conditions() {
    let mut buffer = Buffer::new();

    // Insert into empty buffer
    assert!(buffer.insert_char('A', Position::origin()));

    // Try to insert at invalid position
    assert!(!buffer.insert_char('X', Position { line: 5, column: 0 }));
    assert!(!buffer.insert_char('Y', Position { line: 0, column: 10 }));
}

#[test]
fn test_delete_boundary_conditions() {
    let mut buffer = Buffer::from_str("A");

    // Delete the only character
    assert!(buffer.delete_char_at(Position::origin()));
    assert_eq!(buffer.content(), "");

    // Try to delete from empty buffer
    assert!(!buffer.delete_char_at(Position::origin()));

    // Try to delete at invalid position
    assert!(!buffer.delete_char_at(Position { line: 5, column: 0 }));
}

#[test]
fn test_unicode_characters() {
    let mut buffer = Buffer::new();

    // Insert unicode characters
    assert!(buffer.insert_char('😀', Position::origin()));
    assert!(buffer.insert_char('你', Position { line: 0, column: 1 }));
    assert!(buffer.insert_char('好', Position { line: 0, column: 2 }));

    assert_eq!(buffer.content(), "😀你好");
    assert_eq!(buffer.len_chars(), 3);

    // Delete unicode character
    assert!(buffer.delete_char_at(Position { line: 0, column: 0 }));
    assert_eq!(buffer.content(), "你好");
}

#[test]
fn test_multiline_operations() {
    let mut buffer = Buffer::from_str("Line 1\nLine 2\nLine 3");

    // Insert in middle line
    assert!(buffer.insert_char('X', Position { line: 1, column: 0 }));
    assert_eq!(buffer.get_line(1), Some("XLine 2\n".to_string()));

    // Delete from different line
    assert!(buffer.delete_char_at(Position { line: 2, column: 0 }));
    assert_eq!(buffer.get_line(2), Some("ine 3".to_string()));
}

#[test]
fn test_empty_lines() {
    let buffer = Buffer::from_str("\n\n");
    assert_eq!(buffer.line_count(), 3); // Three lines (empty, empty, empty)
    assert_eq!(buffer.get_line(0), Some("\n".to_string()));
    assert_eq!(buffer.get_line(1), Some("\n".to_string()));
    assert_eq!(buffer.get_line(2), Some("".to_string()));
}

#[test]
fn test_cursor_position_after_operations() {
    let mut buffer = Buffer::from_str("Test");

    // Position should still be valid after insert
    let pos = Position { line: 0, column: 2 };
    buffer.insert_char('X', pos);
    assert_eq!(buffer.content(), "TeXst");

    // Position is now at a different character after insert
    let next_pos = Position { line: 0, column: 3 };
    assert!(buffer.is_valid_position(next_pos));
}

#[test]
fn test_long_lines() {
    let long_text = "a".repeat(1000);
    let buffer = Buffer::from_str(&long_text);

    assert_eq!(buffer.line_len(0), Some(1000));
    assert!(buffer.is_valid_position(Position { line: 0, column: 500 }));
    assert!(buffer.is_valid_position(Position { line: 0, column: 1000 }));
    assert!(!buffer.is_valid_position(Position { line: 0, column: 1001 }));
}

// Navigation command tests

#[test]
fn test_delete_forward() {
    let mut buffer = Buffer::from_str("ABC");

    // Delete character at position 0 (delete 'A')
    assert!(buffer.delete_forward(Position { line: 0, column: 0 }));
    assert_eq!(buffer.content(), "BC");
    assert!(buffer.is_dirty());

    // Delete character at position 1 (delete 'C')
    assert!(buffer.delete_forward(Position { line: 0, column: 1 }));
    assert_eq!(buffer.content(), "B");
}

#[test]
fn test_delete_forward_at_line_end() {
    let mut buffer = Buffer::from_str("Line1\nLine2");

    // Delete newline at end of first line (joins lines)
    assert!(buffer.delete_forward(Position { line: 0, column: 5 }));
    assert_eq!(buffer.content(), "Line1Line2");
}

#[test]
fn test_delete_forward_at_buffer_end() {
    let mut buffer = Buffer::from_str("ABC");

    // Try to delete at end of buffer (should return false)
    assert!(!buffer.delete_forward(Position { line: 0, column: 3 }));
    assert_eq!(buffer.content(), "ABC");
}

#[test]
fn test_get_line_start() {
    let buffer = Buffer::from_str("Hello World");

    // From middle of line
    let start = buffer.get_line_start(Position { line: 0, column: 5 });
    assert_eq!(start, Position { line: 0, column: 0 });

    // Already at start
    let start = buffer.get_line_start(Position { line: 0, column: 0 });
    assert_eq!(start, Position { line: 0, column: 0 });
}

#[test]
fn test_get_line_end() {
    let buffer = Buffer::from_str("Hello");

    // From start of line
    let end = buffer.get_line_end(Position { line: 0, column: 0 });
    assert_eq!(end, Position { line: 0, column: 5 });

    // From middle of line
    let end = buffer.get_line_end(Position { line: 0, column: 2 });
    assert_eq!(end, Position { line: 0, column: 5 });

    // Already at end
    let end = buffer.get_line_end(Position { line: 0, column: 5 });
    assert_eq!(end, Position { line: 0, column: 5 });
}

#[test]
fn test_get_line_end_with_newline() {
    let buffer = Buffer::from_str("Hello\nWorld");

    // End position excludes newline
    let end = buffer.get_line_end(Position { line: 0, column: 2 });
    assert_eq!(end, Position { line: 0, column: 5 });
}

#[test]
fn test_page_up() {
    let buffer = Buffer::from_str("Line1\nLine2\nLine3\nLine4\nLine5");

    // Move up by 2 lines from line 4
    let new_pos = buffer.page_up(Position { line: 4, column: 0 }, 2);
    assert_eq!(new_pos, Position { line: 2, column: 0 });

    // Move up by 3 lines from line 2
    let new_pos = buffer.page_up(Position { line: 2, column: 0 }, 3);
    assert_eq!(new_pos, Position { line: 0, column: 0 });

    // Move up beyond start (clamps to line 0)
    let new_pos = buffer.page_up(Position { line: 1, column: 0 }, 5);
    assert_eq!(new_pos, Position { line: 0, column: 0 });
}

#[test]
fn test_page_up_column_clamping() {
    let buffer = Buffer::from_str("LongLine123\nShort\nLine3");

    // Start at column 10 on long line, move to shorter line
    let new_pos = buffer.page_up(Position { line: 2, column: 10 }, 1);
    // Column should be clamped to length of "Short" (5 characters)
    assert_eq!(new_pos, Position { line: 1, column: 5 });
}

#[test]
fn test_page_down() {
    let buffer = Buffer::from_str("Line1\nLine2\nLine3\nLine4\nLine5");

    // Move down by 2 lines from line 0
    let new_pos = buffer.page_down(Position { line: 0, column: 0 }, 2);
    assert_eq!(new_pos, Position { line: 2, column: 0 });

    // Move down by 3 lines from line 1
    let new_pos = buffer.page_down(Position { line: 1, column: 0 }, 3);
    assert_eq!(new_pos, Position { line: 4, column: 0 });

    // Move down beyond end (clamps to last line)
    let new_pos = buffer.page_down(Position { line: 3, column: 0 }, 5);
    assert_eq!(new_pos, Position { line: 4, column: 0 });
}

#[test]
fn test_page_down_column_clamping() {
    let buffer = Buffer::from_str("LongLine123\nShort\nLine3");

    // Start at column 10 on long line, move to shorter line
    let new_pos = buffer.page_down(Position { line: 0, column: 10 }, 1);
    // Column should be clamped to length of "Short" (5 characters)
    assert_eq!(new_pos, Position { line: 1, column: 5 });
}

#[test]
fn test_page_navigation_empty_buffer() {
    let buffer = Buffer::new();

    // Page up from origin stays at origin
    let new_pos = buffer.page_up(Position::origin(), 10);
    assert_eq!(new_pos, Position::origin());

    // Page down from origin stays at origin (only 1 line in empty buffer)
    let new_pos = buffer.page_down(Position::origin(), 10);
    assert_eq!(new_pos, Position::origin());
}

#[test]
fn test_page_navigation_single_line() {
    let buffer = Buffer::from_str("Single line");

    // Page up/down on single line buffer stays on line 0
    let new_pos = buffer.page_up(Position { line: 0, column: 5 }, 5);
    assert_eq!(new_pos, Position { line: 0, column: 5 });

    let new_pos = buffer.page_down(Position { line: 0, column: 5 }, 5);
    assert_eq!(new_pos, Position { line: 0, column: 5 });
}
