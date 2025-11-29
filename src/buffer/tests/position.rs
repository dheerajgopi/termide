//! Unit tests for Position struct

use crate::buffer::Position;

#[test]
fn test_position_new() {
    let pos = Position::new(5, 10);
    assert_eq!(pos.line, 5);
    assert_eq!(pos.column, 10);
}

#[test]
fn test_position_origin() {
    let pos = Position::origin();
    assert_eq!(pos.line, 0);
    assert_eq!(pos.column, 0);
}

#[test]
fn test_position_equality() {
    let pos1 = Position::new(3, 7);
    let pos2 = Position::new(3, 7);
    let pos3 = Position::new(3, 8);

    assert_eq!(pos1, pos2);
    assert_ne!(pos1, pos3);
}

#[test]
fn test_position_copy() {
    let pos1 = Position::new(2, 4);
    let pos2 = pos1; // Copy

    assert_eq!(pos1, pos2);
    assert_eq!(pos1.line, 2);
    assert_eq!(pos2.line, 2);
}

#[test]
fn test_position_clone() {
    let pos1 = Position::new(1, 3);
    let pos2 = pos1.clone();

    assert_eq!(pos1, pos2);
}
