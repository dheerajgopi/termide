//! Keyboard event handler and command mapper

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::editor::EditorMode;
use super::{EditorCommand, Direction};

/// Handles a keyboard event and returns the corresponding editor command
///
/// This function maps keyboard events to editor commands based on the current
/// editor mode. The same key can produce different commands depending on the mode.
///
/// # Arguments
///
/// * `event` - The keyboard event to handle
/// * `mode` - The current editor mode
///
/// # Returns
///
/// * `Some(EditorCommand)` - If the key is mapped to a command
/// * `None` - If the key is not mapped (ignored)
///
/// # Key Mappings
///
/// ## Special Keys (work in both modes)
/// - `Ctrl+S` → Save
/// - `Ctrl+Q` → Quit
///
/// ## Insert Mode
/// - Printable characters → InsertChar
/// - `Enter` → InsertChar('\n')
/// - `Backspace` → DeleteChar
/// - `Esc` → ChangeMode(Normal)
///
/// ## Normal Mode
/// - `i` → ChangeMode(Insert)
/// - Arrow keys → MoveCursor
///
/// # Examples
///
/// ```
/// use termide::input::{handle_key_event, EditorCommand, Direction};
/// use termide::editor::EditorMode;
/// use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
///
/// // Insert mode: typing 'a'
/// let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
/// let command = handle_key_event(event, EditorMode::Insert);
/// assert_eq!(command, Some(EditorCommand::InsertChar('a')));
///
/// // Insert mode: Enter key
/// let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
/// let command = handle_key_event(event, EditorMode::Insert);
/// assert_eq!(command, Some(EditorCommand::InsertChar('\n')));
///
/// // Insert mode: Backspace
/// let event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
/// let command = handle_key_event(event, EditorMode::Insert);
/// assert_eq!(command, Some(EditorCommand::DeleteChar));
///
/// // Normal mode: 'i' to enter insert mode
/// let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
/// let command = handle_key_event(event, EditorMode::Normal);
/// assert_eq!(command, Some(EditorCommand::ChangeMode(EditorMode::Insert)));
///
/// // Normal mode: Arrow key navigation
/// let event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
/// let command = handle_key_event(event, EditorMode::Normal);
/// assert_eq!(command, Some(EditorCommand::MoveCursor(Direction::Up)));
///
/// // Ctrl+S saves in both modes
/// let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
/// assert_eq!(handle_key_event(event, EditorMode::Insert), Some(EditorCommand::Save));
/// assert_eq!(handle_key_event(event, EditorMode::Normal), Some(EditorCommand::Save));
///
/// // Unmapped keys return None
/// let event = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
/// assert_eq!(handle_key_event(event, EditorMode::Insert), None);
/// ```
pub fn handle_key_event(event: KeyEvent, mode: EditorMode) -> Option<EditorCommand> {
    // Handle special key combinations that work in all modes
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        match event.code {
            KeyCode::Char('s') | KeyCode::Char('S') => return Some(EditorCommand::Save),
            KeyCode::Char('q') | KeyCode::Char('Q') => return Some(EditorCommand::Quit),
            _ => {}
        }
    }

    // Mode-specific key handling
    match mode {
        EditorMode::Insert => handle_insert_mode(event),
        EditorMode::Normal => handle_normal_mode(event),
    }
}

/// Handles key events in Insert mode
fn handle_insert_mode(event: KeyEvent) -> Option<EditorCommand> {
    match event.code {
        // Printable characters - insert at cursor (only if no special modifiers except SHIFT)
        KeyCode::Char(c) if !event.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            Some(EditorCommand::InsertChar(c))
        }

        // Enter key - insert newline
        KeyCode::Enter => Some(EditorCommand::InsertChar('\n')),

        // Backspace - delete character before cursor
        KeyCode::Backspace => Some(EditorCommand::DeleteChar),

        // Escape - switch to Normal mode
        KeyCode::Esc => Some(EditorCommand::ChangeMode(EditorMode::Normal)),

        // Arrow keys - move cursor (also works in insert mode)
        KeyCode::Up => Some(EditorCommand::MoveCursor(Direction::Up)),
        KeyCode::Down => Some(EditorCommand::MoveCursor(Direction::Down)),
        KeyCode::Left => Some(EditorCommand::MoveCursor(Direction::Left)),
        KeyCode::Right => Some(EditorCommand::MoveCursor(Direction::Right)),

        // Unmapped keys - ignore
        _ => None,
    }
}

/// Handles key events in Normal mode
fn handle_normal_mode(event: KeyEvent) -> Option<EditorCommand> {
    match event.code {
        // 'i' key - switch to Insert mode (only if no special modifiers)
        KeyCode::Char('i') if !event.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            Some(EditorCommand::ChangeMode(EditorMode::Insert))
        }

        // Arrow keys - cursor navigation
        KeyCode::Up => Some(EditorCommand::MoveCursor(Direction::Up)),
        KeyCode::Down => Some(EditorCommand::MoveCursor(Direction::Down)),
        KeyCode::Left => Some(EditorCommand::MoveCursor(Direction::Left)),
        KeyCode::Right => Some(EditorCommand::MoveCursor(Direction::Right)),

        // Unmapped keys - ignore
        _ => None,
    }
}
