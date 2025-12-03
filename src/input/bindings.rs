//! Default keybindings for TermIDE
//!
//! This module defines all default keybindings organized by mode and context.
//! It serves as the single source of truth for all built-in editor keybindings.
//!
//! # Organization
//!
//! Bindings are organized into logical groups:
//! - **Global bindings**: Active in Insert and Normal modes (Ctrl+S, Ctrl+Q)
//! - **Insert mode bindings**: Character insertion, Enter, Backspace, Esc
//! - **Normal mode bindings**: Mode switching ('i'), navigation
//! - **Prompt mode bindings**: Prompt input handling
//! - **Common navigation**: Arrow keys shared across multiple modes
//!
//! # Usage
//!
//! ```no_run
//! use termide::input::bindings::register_default_bindings;
//! use termide::input::registry::KeyBindingRegistry;
//! use std::time::Duration;
//!
//! let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
//! register_default_bindings(&mut registry).expect("default bindings should register");
//! ```

use crate::editor::EditorMode;
use crate::input::keybinding::{
    BindingContext, KeyBinding, KeyPattern, KeySequence, Priority, PRIMARY_MODIFIER,
};
use crate::input::registry::{BindingError, KeyBindingRegistry};
use crate::input::{Direction, EditorCommand};
use crossterm::event::{KeyCode, KeyModifiers};

/// Registers all default keybindings into the provided registry
///
/// This function loads all built-in editor keybindings organized by mode and context.
/// It should be called during editor initialization to populate the registry with
/// default bindings before loading user customizations.
///
/// # Arguments
///
/// * `registry` - The keybinding registry to populate
///
/// # Returns
///
/// - `Ok(())` if all bindings registered successfully
/// - `Err(BindingError)` if any binding conflicts (should not happen with defaults)
///
/// # Examples
///
/// ```no_run
/// use termide::input::bindings::register_default_bindings;
/// use termide::input::registry::KeyBindingRegistry;
/// use std::time::Duration;
///
/// let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
/// register_default_bindings(&mut registry).expect("defaults should register");
/// assert!(registry.len() > 0);
/// ```
pub fn register_default_bindings(registry: &mut KeyBindingRegistry) -> Result<(), BindingError> {
    // Register global bindings (active in Insert and Normal modes)
    for binding in global_bindings() {
        registry.register(binding)?;
    }

    // Register Insert mode specific bindings
    for binding in insert_mode_bindings() {
        registry.register(binding)?;
    }

    // Register Normal mode specific bindings
    for binding in normal_mode_bindings() {
        registry.register(binding)?;
    }

    // Register Prompt mode specific bindings
    for binding in prompt_mode_bindings() {
        registry.register(binding)?;
    }

    Ok(())
}

/// Returns global keybindings active in Insert and Normal modes
///
/// Global bindings are universal shortcuts like Ctrl+S (save) and Ctrl+Q (quit)
/// that should work consistently across editing modes but not in Prompt mode.
///
/// # Bindings
///
/// - `Ctrl+S` / `Cmd+S` → Save file
/// - `Ctrl+Q` / `Cmd+Q` → Quit editor
///
/// # Examples
///
/// ```
/// use termide::input::bindings::global_bindings;
///
/// let bindings = global_bindings();
/// assert_eq!(bindings.len(), 4);
/// ```
pub fn global_bindings() -> Vec<KeyBinding> {
    vec![
        // Save file (Ctrl+S on Linux/Windows, Cmd+S on macOS)
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(
                KeyCode::Char('s'),
                PRIMARY_MODIFIER,
            )])
            .expect("Ctrl+S is valid"),
            EditorCommand::Save,
            BindingContext::Global,
            Priority::Default,
        ),
        // Alternative: uppercase 'S' with modifier
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(
                KeyCode::Char('S'),
                PRIMARY_MODIFIER,
            )])
            .expect("Ctrl+Shift+S is valid"),
            EditorCommand::Save,
            BindingContext::Global,
            Priority::Default,
        ),
        // Quit editor (Ctrl+Q on Linux/Windows, Cmd+Q on macOS)
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(
                KeyCode::Char('q'),
                PRIMARY_MODIFIER,
            )])
            .expect("Ctrl+Q is valid"),
            EditorCommand::Quit,
            BindingContext::Global,
            Priority::Default,
        ),
        // Alternative: uppercase 'Q' with modifier
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(
                KeyCode::Char('Q'),
                PRIMARY_MODIFIER,
            )])
            .expect("Ctrl+Shift+Q is valid"),
            EditorCommand::Quit,
            BindingContext::Global,
            Priority::Default,
        ),
    ]
}

/// Returns Insert mode specific keybindings
///
/// These bindings handle character insertion, special keys, and mode switching
/// when in Insert mode.
///
/// # Bindings
///
/// - Printable characters → Insert at cursor
/// - `Enter` → Insert newline
/// - `Backspace` → Delete character before cursor
/// - `Esc` → Switch to Normal mode
/// - Arrow keys → Move cursor (shared with Normal mode)
///
/// # Examples
///
/// ```
/// use termide::input::bindings::insert_mode_bindings;
///
/// let bindings = insert_mode_bindings();
/// assert!(bindings.len() > 0);
/// ```
pub fn insert_mode_bindings() -> Vec<KeyBinding> {
    let mut bindings = Vec::new();

    // Enter key - insert newline
    bindings.push(KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Enter, KeyModifiers::NONE)])
            .expect("Enter is valid"),
        EditorCommand::InsertChar('\n'),
        BindingContext::Mode(EditorMode::Insert),
        Priority::Default,
    ));

    // Backspace - delete character before cursor
    bindings.push(KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(
            KeyCode::Backspace,
            KeyModifiers::NONE,
        )])
        .expect("Backspace is valid"),
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Insert),
        Priority::Default,
    ));

    // Escape - switch to Normal mode
    bindings.push(KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(KeyCode::Esc, KeyModifiers::NONE)])
            .expect("Esc is valid"),
        EditorCommand::ChangeMode(EditorMode::Normal),
        BindingContext::Mode(EditorMode::Insert),
        Priority::Default,
    ));

    // Arrow key navigation (shared with Normal mode)
    bindings.extend(arrow_key_navigation(vec![
        EditorMode::Insert,
        EditorMode::Normal,
    ]));

    // Printable characters - handled dynamically by InputHandler
    // We don't register individual character bindings as that would create
    // thousands of entries. Instead, the handler checks for printable chars.

    bindings
}

/// Returns Normal mode specific keybindings
///
/// These bindings handle vim-style navigation and mode switching when in Normal mode.
///
/// # Bindings
///
/// - `i` → Switch to Insert mode
/// - Arrow keys → Move cursor (shared with Insert mode)
///
/// # Examples
///
/// ```
/// use termide::input::bindings::normal_mode_bindings;
///
/// let bindings = normal_mode_bindings();
/// assert!(bindings.len() > 0);
/// ```
pub fn normal_mode_bindings() -> Vec<KeyBinding> {
    let mut bindings = Vec::new();

    // 'i' key - switch to Insert mode
    bindings.push(KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(
            KeyCode::Char('i'),
            KeyModifiers::NONE,
        )])
        .expect("i is valid"),
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    ));

    // Arrow key navigation is already added via insert_mode_bindings
    // with Modes([Insert, Normal]) context, so we don't duplicate it here

    bindings
}

/// Returns Prompt mode specific keybindings
///
/// These bindings handle prompt input when the editor is collecting user input
/// for commands, searches, or other prompts.
///
/// # Bindings
///
/// - Printable characters → Insert into prompt
/// - `Backspace` → Delete character from prompt
/// - `Enter` → Accept prompt input
/// - `Esc` → Cancel prompt
///
/// # Examples
///
/// ```
/// use termide::input::bindings::prompt_mode_bindings;
///
/// let bindings = prompt_mode_bindings();
/// assert!(bindings.len() > 0);
/// ```
pub fn prompt_mode_bindings() -> Vec<KeyBinding> {
    vec![
        // Backspace - delete character from prompt input
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(
                KeyCode::Backspace,
                KeyModifiers::NONE,
            )])
            .expect("Backspace is valid"),
            EditorCommand::PromptDeleteChar,
            BindingContext::Mode(EditorMode::Prompt),
            Priority::Default,
        ),
        // Enter - accept prompt input
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(KeyCode::Enter, KeyModifiers::NONE)])
                .expect("Enter is valid"),
            EditorCommand::AcceptPrompt,
            BindingContext::Mode(EditorMode::Prompt),
            Priority::Default,
        ),
        // Escape - cancel prompt
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(KeyCode::Esc, KeyModifiers::NONE)])
                .expect("Esc is valid"),
            EditorCommand::CancelPrompt,
            BindingContext::Mode(EditorMode::Prompt),
            Priority::Default,
        ),
        // Printable characters handled dynamically by InputHandler
    ]
}

/// Returns arrow key navigation bindings for the specified modes
///
/// This helper function creates arrow key bindings for cursor movement,
/// avoiding duplication by defining them once and reusing across modes.
///
/// # Arguments
///
/// * `modes` - Vector of modes in which these bindings should be active
///
/// # Bindings
///
/// - `Up` → Move cursor up
/// - `Down` → Move cursor down
/// - `Left` → Move cursor left
/// - `Right` → Move cursor right
///
/// # Examples
///
/// ```
/// use termide::input::bindings::arrow_key_navigation;
/// use termide::editor::EditorMode;
///
/// // Create navigation bindings for Insert and Normal modes
/// let bindings = arrow_key_navigation(vec![EditorMode::Insert, EditorMode::Normal]);
/// assert_eq!(bindings.len(), 4);
/// ```
pub fn arrow_key_navigation(modes: Vec<EditorMode>) -> Vec<KeyBinding> {
    let context = BindingContext::Modes(modes);

    vec![
        // Up arrow
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(KeyCode::Up, KeyModifiers::NONE)])
                .expect("Up is valid"),
            EditorCommand::MoveCursor(Direction::Up),
            context.clone(),
            Priority::Default,
        ),
        // Down arrow
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(KeyCode::Down, KeyModifiers::NONE)])
                .expect("Down is valid"),
            EditorCommand::MoveCursor(Direction::Down),
            context.clone(),
            Priority::Default,
        ),
        // Left arrow
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(KeyCode::Left, KeyModifiers::NONE)])
                .expect("Left is valid"),
            EditorCommand::MoveCursor(Direction::Left),
            context.clone(),
            Priority::Default,
        ),
        // Right arrow
        KeyBinding::new(
            KeySequence::new(vec![KeyPattern::new(KeyCode::Right, KeyModifiers::NONE)])
                .expect("Right is valid"),
            EditorCommand::MoveCursor(Direction::Right),
            context,
            Priority::Default,
        ),
    ]
}
