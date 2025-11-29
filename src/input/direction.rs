//! Cursor movement direction enumeration

/// Represents the direction of cursor movement
///
/// Used in conjunction with the `MoveCursor` command to specify
/// which direction the cursor should move.
///
/// # Examples
///
/// ```
/// use termide::input::Direction;
///
/// let dir = Direction::Up;
/// let dir = Direction::Down;
/// let dir = Direction::Left;
/// let dir = Direction::Right;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Move cursor up one line
    Up,

    /// Move cursor down one line
    Down,

    /// Move cursor left one character
    Left,

    /// Move cursor right one character
    Right,
}
