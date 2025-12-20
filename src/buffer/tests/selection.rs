//! Unit tests for Selection struct

use crate::buffer::{Position, Selection};

#[test]
fn test_new_creates_collapsed_selection() {
    let pos = Position::new(2, 5);
    let sel = Selection::new(pos);

    assert_eq!(sel.anchor(), pos);
    assert_eq!(sel.cursor(), pos);
    assert!(!sel.has_selection());
}

#[test]
fn test_new_at_origin() {
    let sel = Selection::new(Position::origin());

    assert_eq!(sel.anchor(), Position::origin());
    assert_eq!(sel.cursor(), Position::origin());
    assert!(!sel.has_selection());
}

#[test]
fn test_with_anchor_and_cursor_forward() {
    let anchor = Position::new(0, 0);
    let cursor = Position::new(0, 5);
    let sel = Selection::with_anchor_and_cursor(anchor, cursor);

    assert_eq!(sel.anchor(), anchor);
    assert_eq!(sel.cursor(), cursor);
    assert!(sel.has_selection());
}

#[test]
fn test_with_anchor_and_cursor_backward() {
    let anchor = Position::new(0, 5);
    let cursor = Position::new(0, 0);
    let sel = Selection::with_anchor_and_cursor(anchor, cursor);

    assert_eq!(sel.anchor(), anchor);
    assert_eq!(sel.cursor(), cursor);
    assert!(sel.has_selection());
}

#[test]
fn test_with_anchor_and_cursor_multiline_forward() {
    let anchor = Position::new(0, 0);
    let cursor = Position::new(2, 10);
    let sel = Selection::with_anchor_and_cursor(anchor, cursor);

    assert_eq!(sel.anchor(), anchor);
    assert_eq!(sel.cursor(), cursor);
    assert!(sel.has_selection());
}

#[test]
fn test_with_anchor_and_cursor_multiline_backward() {
    let anchor = Position::new(2, 10);
    let cursor = Position::new(0, 0);
    let sel = Selection::with_anchor_and_cursor(anchor, cursor);

    assert_eq!(sel.anchor(), anchor);
    assert_eq!(sel.cursor(), cursor);
    assert!(sel.has_selection());
}

#[test]
fn test_with_anchor_and_cursor_same_position() {
    let pos = Position::new(1, 3);
    let sel = Selection::with_anchor_and_cursor(pos, pos);

    assert_eq!(sel.anchor(), pos);
    assert_eq!(sel.cursor(), pos);
    assert!(!sel.has_selection());
}

#[test]
fn test_has_selection_true_when_different_positions() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));
    assert!(sel.has_selection());
}

#[test]
fn test_has_selection_false_when_collapsed() {
    let sel = Selection::new(Position::new(1, 1));
    assert!(!sel.has_selection());
}

#[test]
fn test_range_forward_selection() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));
    let (start, end) = sel.range();

    assert_eq!(start, Position::new(0, 0));
    assert_eq!(end, Position::new(0, 5));
}

#[test]
fn test_range_backward_selection() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 5), Position::new(0, 0));
    let (start, end) = sel.range();

    assert_eq!(start, Position::new(0, 0));
    assert_eq!(end, Position::new(0, 5));
}

#[test]
fn test_range_multiline_forward() {
    let sel = Selection::with_anchor_and_cursor(Position::new(1, 2), Position::new(3, 4));
    let (start, end) = sel.range();

    assert_eq!(start, Position::new(1, 2));
    assert_eq!(end, Position::new(3, 4));
}

#[test]
fn test_range_multiline_backward() {
    let sel = Selection::with_anchor_and_cursor(Position::new(3, 4), Position::new(1, 2));
    let (start, end) = sel.range();

    assert_eq!(start, Position::new(1, 2));
    assert_eq!(end, Position::new(3, 4));
}

#[test]
fn test_range_collapsed_selection() {
    let pos = Position::new(2, 5);
    let sel = Selection::new(pos);
    let (start, end) = sel.range();

    assert_eq!(start, pos);
    assert_eq!(end, pos);
}

#[test]
fn test_contains_position_within_selection() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));

    assert!(sel.contains(Position::new(0, 0))); // inclusive start
    assert!(sel.contains(Position::new(0, 1)));
    assert!(sel.contains(Position::new(0, 3)));
    assert!(sel.contains(Position::new(0, 4)));
}

#[test]
fn test_contains_position_at_end_exclusive() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));
    assert!(!sel.contains(Position::new(0, 5))); // exclusive end
}

#[test]
fn test_contains_position_outside_selection() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));

    assert!(!sel.contains(Position::new(0, 6)));
    assert!(!sel.contains(Position::new(1, 0)));
    assert!(!sel.contains(Position::new(1, 3)));
}

#[test]
fn test_contains_position_backward_selection() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 5), Position::new(0, 0));

    assert!(sel.contains(Position::new(0, 0))); // inclusive start
    assert!(sel.contains(Position::new(0, 3)));
    assert!(!sel.contains(Position::new(0, 5))); // exclusive end
}

#[test]
fn test_contains_multiline_selection() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 5), Position::new(2, 3));

    assert!(sel.contains(Position::new(0, 5))); // start line
    assert!(sel.contains(Position::new(0, 10))); // start line, after start column
    assert!(sel.contains(Position::new(1, 0))); // middle line
    assert!(sel.contains(Position::new(1, 100))); // middle line, any column
    assert!(sel.contains(Position::new(2, 0))); // end line, before end column
    assert!(sel.contains(Position::new(2, 2))); // end line, before end column
    assert!(!sel.contains(Position::new(2, 3))); // end line, at end column (exclusive)
    assert!(!sel.contains(Position::new(2, 5))); // end line, after end column
    assert!(!sel.contains(Position::new(3, 0))); // after end line
}

#[test]
fn test_contains_collapsed_selection() {
    let sel = Selection::new(Position::new(1, 5));
    assert!(!sel.contains(Position::new(1, 5)));
    assert!(!sel.contains(Position::new(0, 0)));
    assert!(!sel.contains(Position::new(2, 0)));
}

#[test]
fn test_extend_to_forward() {
    let sel = Selection::new(Position::new(0, 0));
    let extended = sel.extend_to(Position::new(0, 5));

    assert_eq!(extended.anchor(), Position::new(0, 0));
    assert_eq!(extended.cursor(), Position::new(0, 5));
    assert!(extended.has_selection());
}

#[test]
fn test_extend_to_backward() {
    let sel = Selection::new(Position::new(0, 5));
    let extended = sel.extend_to(Position::new(0, 0));

    assert_eq!(extended.anchor(), Position::new(0, 5));
    assert_eq!(extended.cursor(), Position::new(0, 0));
    assert!(extended.has_selection());
}

#[test]
fn test_extend_to_multiline() {
    let sel = Selection::new(Position::new(0, 0));
    let extended = sel.extend_to(Position::new(3, 10));

    assert_eq!(extended.anchor(), Position::new(0, 0));
    assert_eq!(extended.cursor(), Position::new(3, 10));
    assert!(extended.has_selection());
}

#[test]
fn test_extend_to_multiple_times() {
    let sel = Selection::new(Position::new(0, 0));
    let sel = sel.extend_to(Position::new(0, 5));
    let sel = sel.extend_to(Position::new(0, 10));
    let sel = sel.extend_to(Position::new(1, 2));

    assert_eq!(sel.anchor(), Position::new(0, 0));
    assert_eq!(sel.cursor(), Position::new(1, 2));
    assert!(sel.has_selection());
}

#[test]
fn test_extend_to_collapse_when_same_as_anchor() {
    let sel = Selection::new(Position::new(0, 0));
    let extended = sel.extend_to(Position::new(0, 5));
    let collapsed = extended.extend_to(Position::new(0, 0));

    assert_eq!(collapsed.anchor(), Position::new(0, 0));
    assert_eq!(collapsed.cursor(), Position::new(0, 0));
    assert!(!collapsed.has_selection());
}

#[test]
fn test_clear_forward_selection() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));
    let cleared = sel.clear();

    assert_eq!(cleared.anchor(), Position::new(0, 5));
    assert_eq!(cleared.cursor(), Position::new(0, 5));
    assert!(!cleared.has_selection());
}

#[test]
fn test_clear_backward_selection() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 5), Position::new(0, 0));
    let cleared = sel.clear();

    assert_eq!(cleared.anchor(), Position::new(0, 0));
    assert_eq!(cleared.cursor(), Position::new(0, 0));
    assert!(!cleared.has_selection());
}

#[test]
fn test_clear_collapsed_selection() {
    let sel = Selection::new(Position::new(2, 3));
    let cleared = sel.clear();

    assert_eq!(cleared.anchor(), Position::new(2, 3));
    assert_eq!(cleared.cursor(), Position::new(2, 3));
    assert!(!cleared.has_selection());
}

#[test]
fn test_clear_multiline_selection() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(5, 10));
    let cleared = sel.clear();

    assert_eq!(cleared.anchor(), Position::new(5, 10));
    assert_eq!(cleared.cursor(), Position::new(5, 10));
    assert!(!cleared.has_selection());
}

#[test]
fn test_selection_equality() {
    let sel1 = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));
    let sel2 = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));
    let sel3 = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 6));

    assert_eq!(sel1, sel2);
    assert_ne!(sel1, sel3);
}

#[test]
fn test_selection_clone() {
    let sel1 = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));
    let sel2 = sel1.clone();

    assert_eq!(sel1, sel2);
}

#[test]
fn test_selection_copy_semantics() {
    let sel1 = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));
    let sel2 = sel1; // Copy, not move
    let sel3 = sel1; // Can still use sel1

    assert_eq!(sel1, sel2);
    assert_eq!(sel1, sel3);
    assert_eq!(sel2, sel3);
}

#[test]
fn test_selection_debug_format() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 5));
    let debug_str = format!("{:?}", sel);

    assert!(debug_str.contains("Selection"));
    assert!(debug_str.contains("anchor"));
    assert!(debug_str.contains("cursor"));
}

#[test]
fn test_edge_case_large_positions() {
    let sel = Selection::with_anchor_and_cursor(
        Position::new(1000, 500),
        Position::new(2000, 1000),
    );

    assert!(sel.has_selection());
    assert_eq!(sel.range(), (Position::new(1000, 500), Position::new(2000, 1000)));
    assert!(sel.contains(Position::new(1500, 0)));
}

#[test]
fn test_boundary_single_character_selection() {
    let sel = Selection::with_anchor_and_cursor(Position::new(0, 0), Position::new(0, 1));

    assert!(sel.has_selection());
    assert!(sel.contains(Position::new(0, 0)));
    assert!(!sel.contains(Position::new(0, 1)));
}

#[test]
fn test_boundary_single_line_full_selection() {
    // Selecting from start to some position on same line
    let sel = Selection::with_anchor_and_cursor(Position::new(5, 0), Position::new(5, 100));

    assert!(sel.has_selection());
    assert!(sel.contains(Position::new(5, 0)));
    assert!(sel.contains(Position::new(5, 50)));
    assert!(!sel.contains(Position::new(5, 100)));
    assert!(!sel.contains(Position::new(4, 0)));
    assert!(!sel.contains(Position::new(6, 0)));
}

#[test]
fn test_memory_footprint() {
    use std::mem::size_of;

    // Selection should be 24 bytes on 64-bit systems (2 × Position)
    // Each Position is 2 × usize = 2 × 8 = 16 bytes
    // Selection is 2 × Position = 2 × 16 = 32 bytes on 64-bit, or 16 on 32-bit
    let selection_size = size_of::<Selection>();
    let position_size = size_of::<Position>();

    assert_eq!(selection_size, 2 * position_size);
}
