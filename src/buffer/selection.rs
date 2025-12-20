//! Text selection support for the buffer module.
//!
//! This module provides the `Selection` struct for representing text selections
//! with anchor and cursor positions. Selections can be forward (cursor after anchor)
//! or backward (cursor before anchor), and the module handles both cases transparently.

use super::Position;

/// Represents a text selection with an anchor and cursor position.
///
/// The anchor is the fixed starting point where the selection began,
/// and the cursor is the movable endpoint that extends or shrinks the selection.
/// When anchor equals cursor, the selection is "collapsed" (no text selected).
///
/// # Examples
///
/// ```
/// use termide::buffer::{Selection, Position};
///
/// // Create a collapsed selection at the origin
/// let sel = Selection::new(Position::origin());
/// assert!(!sel.has_selection());
///
/// // Extend the selection
/// let sel = sel.extend_to(Position::new(0, 5));
/// assert!(sel.has_selection());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// The fixed starting point of the selection
    anchor: Position,
    /// The movable endpoint of the selection
    cursor: Position,
}

impl Selection {
    /// Creates a new collapsed selection at the given position.
    ///
    /// A collapsed selection has the anchor and cursor at the same position,
    /// meaning no text is selected.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Selection, Position};
    ///
    /// let sel = Selection::new(Position::new(2, 5));
    /// assert_eq!(sel.anchor(), Position::new(2, 5));
    /// assert_eq!(sel.cursor(), Position::new(2, 5));
    /// assert!(!sel.has_selection());
    /// ```
    pub fn new(pos: Position) -> Self {
        Self {
            anchor: pos,
            cursor: pos,
        }
    }

    /// Creates a selection with explicit anchor and cursor positions.
    ///
    /// This allows creating selections in either direction (forward or backward).
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Selection, Position};
    ///
    /// // Forward selection
    /// let sel = Selection::with_anchor_and_cursor(
    ///     Position::new(0, 0),
    ///     Position::new(0, 5)
    /// );
    /// assert_eq!(sel.range(), (Position::new(0, 0), Position::new(0, 5)));
    ///
    /// // Backward selection
    /// let sel = Selection::with_anchor_and_cursor(
    ///     Position::new(0, 5),
    ///     Position::new(0, 0)
    /// );
    /// assert_eq!(sel.range(), (Position::new(0, 0), Position::new(0, 5)));
    /// ```
    pub fn with_anchor_and_cursor(anchor: Position, cursor: Position) -> Self {
        Self { anchor, cursor }
    }

    /// Returns the anchor position.
    ///
    /// The anchor is the fixed starting point where the selection began.
    pub fn anchor(&self) -> Position {
        self.anchor
    }

    /// Returns the cursor position.
    ///
    /// The cursor is the movable endpoint of the selection.
    pub fn cursor(&self) -> Position {
        self.cursor
    }

    /// Returns true if this selection has any text selected.
    ///
    /// Returns false if the selection is collapsed (anchor equals cursor).
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Selection, Position};
    ///
    /// let sel = Selection::new(Position::origin());
    /// assert!(!sel.has_selection());
    ///
    /// let sel = Selection::with_anchor_and_cursor(
    ///     Position::origin(),
    ///     Position::new(0, 5)
    /// );
    /// assert!(sel.has_selection());
    /// ```
    pub fn has_selection(&self) -> bool {
        self.anchor != self.cursor
    }

    /// Returns the normalized range of the selection as (start, end).
    ///
    /// The returned positions are ordered such that start <= end, regardless
    /// of the selection direction (forward or backward).
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Selection, Position};
    ///
    /// // Forward selection
    /// let sel = Selection::with_anchor_and_cursor(
    ///     Position::new(0, 0),
    ///     Position::new(0, 5)
    /// );
    /// assert_eq!(sel.range(), (Position::new(0, 0), Position::new(0, 5)));
    ///
    /// // Backward selection returns same range
    /// let sel = Selection::with_anchor_and_cursor(
    ///     Position::new(0, 5),
    ///     Position::new(0, 0)
    /// );
    /// assert_eq!(sel.range(), (Position::new(0, 0), Position::new(0, 5)));
    /// ```
    pub fn range(&self) -> (Position, Position) {
        if self.cursor >= self.anchor {
            (self.anchor, self.cursor)
        } else {
            (self.cursor, self.anchor)
        }
    }

    /// Returns true if the given position is within the selection bounds.
    ///
    /// The check is inclusive of the start position but exclusive of the end position.
    /// This matches standard text editor selection behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Selection, Position};
    ///
    /// let sel = Selection::with_anchor_and_cursor(
    ///     Position::new(0, 0),
    ///     Position::new(0, 5)
    /// );
    ///
    /// assert!(sel.contains(Position::new(0, 0))); // inclusive start
    /// assert!(sel.contains(Position::new(0, 3))); // within range
    /// assert!(!sel.contains(Position::new(0, 5))); // exclusive end
    /// assert!(!sel.contains(Position::new(1, 0))); // different line
    /// ```
    pub fn contains(&self, pos: Position) -> bool {
        let (start, end) = self.range();
        pos >= start && pos < end
    }

    /// Extends the selection to the given position by moving the cursor.
    ///
    /// The anchor remains fixed, and only the cursor moves to the new position.
    /// This is the typical behavior when holding Shift and moving the cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Selection, Position};
    ///
    /// let sel = Selection::new(Position::new(0, 0));
    /// let sel = sel.extend_to(Position::new(0, 5));
    ///
    /// assert_eq!(sel.anchor(), Position::new(0, 0));
    /// assert_eq!(sel.cursor(), Position::new(0, 5));
    /// assert!(sel.has_selection());
    /// ```
    pub fn extend_to(self, pos: Position) -> Self {
        Self {
            anchor: self.anchor,
            cursor: pos,
        }
    }

    /// Clears the selection by collapsing it to the cursor position.
    ///
    /// After calling this method, the anchor will equal the cursor,
    /// and `has_selection()` will return false.
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::buffer::{Selection, Position};
    ///
    /// let sel = Selection::with_anchor_and_cursor(
    ///     Position::new(0, 0),
    ///     Position::new(0, 5)
    /// );
    /// assert!(sel.has_selection());
    ///
    /// let sel = sel.clear();
    /// assert!(!sel.has_selection());
    /// assert_eq!(sel.anchor(), sel.cursor());
    /// ```
    pub fn clear(self) -> Self {
        Self {
            anchor: self.cursor,
            cursor: self.cursor,
        }
    }
}
