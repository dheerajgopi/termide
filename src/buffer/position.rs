//! Position struct for tracking cursor and text locations in the buffer

/// Represents a position in the buffer with 0-indexed line and column
///
/// # Examples
///
/// ```
/// use termide::buffer::Position;
///
/// let pos = Position::new(5, 10);
/// assert_eq!(pos.line, 5);
/// assert_eq!(pos.column, 10);
///
/// let origin = Position::origin();
/// assert_eq!(origin.line, 0);
/// assert_eq!(origin.column, 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed, character-based not byte-based)
    pub column: usize,
}

impl Position {
    /// Creates a new Position
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::Position;
    ///
    /// let pos = Position::new(10, 25);
    /// assert_eq!(pos.line, 10);
    /// assert_eq!(pos.column, 25);
    /// ```
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Creates a Position at the origin (0, 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::Position;
    ///
    /// let pos = Position::origin();
    /// assert_eq!(pos.line, 0);
    /// assert_eq!(pos.column, 0);
    /// ```
    pub fn origin() -> Self {
        Self { line: 0, column: 0 }
    }
}
