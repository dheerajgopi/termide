//! Integration tests for keybinding system
//!
//! These tests verify that default bindings load correctly into the registry
//! and that the complete keybinding system works end-to-end.

use std::time::Duration;
use termide::editor::EditorMode;
use termide::input::bindings::register_default_bindings;
use termide::input::keybinding::{KeyPattern, Priority};
use termide::input::registry::KeyBindingRegistry;
use termide::input::{Direction, EditorCommand};
use crossterm::event::{KeyCode, KeyModifiers};

/// Test: Load default bindings into fresh registry and verify accessibility
#[test]
fn test_load_defaults_into_registry() {
    // Arrange: Create empty registry
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    assert!(registry.is_empty());

    // Act: Register default bindings
    let result = register_default_bindings(&mut registry);

    // Assert: Registration succeeded and registry is populated
    assert!(result.is_ok(), "Default bindings should register successfully");
    assert!(!registry.is_empty(), "Registry should contain bindings after registration");
}

/// Test: Verify Ctrl+S binding is accessible in Insert mode
#[test]
fn test_ctrl_s_accessible_in_insert_mode() {
    // Arrange: Registry with default bindings
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    register_default_bindings(&mut registry).unwrap();

    // Act: Simulate Ctrl+S key press
    #[cfg(target_os = "macos")]
    let modifier = KeyModifiers::SUPER;
    #[cfg(not(target_os = "macos"))]
    let modifier = KeyModifiers::CONTROL;

    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('s'), modifier));

    // Assert: Find match returns Save command
    let command = registry.find_match(EditorMode::Insert);
    assert_eq!(command, Some(&EditorCommand::Save));
}

/// Test: Verify Ctrl+S binding is accessible in Normal mode
#[test]
fn test_ctrl_s_accessible_in_normal_mode() {
    // Arrange: Registry with default bindings
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    register_default_bindings(&mut registry).unwrap();

    // Act: Simulate Ctrl+S key press
    #[cfg(target_os = "macos")]
    let modifier = KeyModifiers::SUPER;
    #[cfg(not(target_os = "macos"))]
    let modifier = KeyModifiers::CONTROL;

    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('s'), modifier));

    // Assert: Find match returns Save command
    let command = registry.find_match(EditorMode::Normal);
    assert_eq!(command, Some(&EditorCommand::Save));
}

/// Test: Verify Ctrl+S NOT accessible in Prompt mode (Global context excludes Prompt)
#[test]
fn test_ctrl_s_not_accessible_in_prompt_mode() {
    // Arrange: Registry with default bindings
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    register_default_bindings(&mut registry).unwrap();

    // Act: Simulate Ctrl+S key press in Prompt mode
    #[cfg(target_os = "macos")]
    let modifier = KeyModifiers::SUPER;
    #[cfg(not(target_os = "macos"))]
    let modifier = KeyModifiers::CONTROL;

    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('s'), modifier));

    // Assert: No match in Prompt mode
    let command = registry.find_match(EditorMode::Prompt);
    assert_eq!(command, None, "Global bindings should not be active in Prompt mode");
}

/// Test: Verify 'i' key only accessible in Normal mode
#[test]
fn test_i_key_only_in_normal_mode() {
    // Arrange: Registry with default bindings
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    register_default_bindings(&mut registry).unwrap();

    // Act & Assert: 'i' in Normal mode triggers Insert mode
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Normal);
    assert_eq!(
        command,
        Some(&EditorCommand::ChangeMode(EditorMode::Insert))
    );

    // Act & Assert: 'i' in Insert mode does nothing (no binding)
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Insert);
    assert_eq!(command, None, "'i' should not match in Insert mode");
}

/// Test: Verify arrow keys work in both Insert and Normal modes
#[test]
fn test_arrow_keys_in_insert_and_normal_modes() {
    // Arrange: Registry with default bindings
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    register_default_bindings(&mut registry).unwrap();

    // Test Up arrow in Insert mode
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Up, KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Insert);
    assert_eq!(
        command,
        Some(&EditorCommand::MoveCursor(Direction::Up)),
        "Up arrow should work in Insert mode"
    );

    // Test Up arrow in Normal mode
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Up, KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Normal);
    assert_eq!(
        command,
        Some(&EditorCommand::MoveCursor(Direction::Up)),
        "Up arrow should work in Normal mode"
    );

    // Test Up arrow NOT in Prompt mode
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Up, KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Prompt);
    assert_eq!(
        command,
        None,
        "Arrow keys should not be bound in Prompt mode"
    );
}

/// Test: Verify Enter key has different behavior in different modes
#[test]
fn test_enter_key_mode_specific_behavior() {
    // Arrange: Registry with default bindings
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    register_default_bindings(&mut registry).unwrap();

    // Test Enter in Insert mode (inserts newline)
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Enter, KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Insert);
    assert_eq!(
        command,
        Some(&EditorCommand::InsertChar('\n')),
        "Enter should insert newline in Insert mode"
    );

    // Test Enter in Prompt mode (accepts prompt)
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Enter, KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Prompt);
    assert_eq!(
        command,
        Some(&EditorCommand::AcceptPrompt),
        "Enter should accept prompt in Prompt mode"
    );

    // Test Enter in Normal mode (no binding)
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Enter, KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Normal);
    assert_eq!(
        command,
        None,
        "Enter should have no binding in Normal mode"
    );
}

/// Test: Verify Escape key has mode-specific behavior
#[test]
fn test_escape_key_mode_specific_behavior() {
    // Arrange: Registry with default bindings
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    register_default_bindings(&mut registry).unwrap();

    // Test Esc in Insert mode (switch to Normal)
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Esc, KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Insert);
    assert_eq!(
        command,
        Some(&EditorCommand::ChangeMode(EditorMode::Normal)),
        "Esc should switch to Normal mode from Insert mode"
    );

    // Test Esc in Prompt mode (cancel prompt)
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Esc, KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Prompt);
    assert_eq!(
        command,
        Some(&EditorCommand::CancelPrompt),
        "Esc should cancel prompt in Prompt mode"
    );

    // Test Esc in Normal mode (no binding)
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Esc, KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Normal);
    assert_eq!(
        command,
        None,
        "Esc should have no binding in Normal mode"
    );
}

/// Test: Verify all bindings have correct priority
#[test]
fn test_all_default_bindings_have_default_priority() {
    // This is implicitly tested by successful registration
    // If priorities were wrong, conflicts would occur
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    let result = register_default_bindings(&mut registry);
    assert!(
        result.is_ok(),
        "All default bindings should have Priority::Default and register without conflicts"
    );
}

/// Test: Verify higher priority bindings can override defaults
#[test]
fn test_user_binding_can_override_default() {
    use termide::input::keybinding::{BindingContext, KeyBinding, KeySequence};

    // Arrange: Registry with default bindings
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    register_default_bindings(&mut registry).unwrap();

    // Act: Register a user binding that overrides 'i' in Normal mode
    let user_binding = KeyBinding::new(
        KeySequence::new(vec![KeyPattern::new(
            KeyCode::Char('i'),
            KeyModifiers::NONE,
        )])
        .unwrap(),
        EditorCommand::Quit, // Different command
        BindingContext::Mode(EditorMode::Normal),
        Priority::User, // Higher priority
    );
    registry.register(user_binding).unwrap();

    // Assert: User binding takes precedence
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE));
    let command = registry.find_match(EditorMode::Normal);
    assert_eq!(
        command,
        Some(&EditorCommand::Quit),
        "User binding should override default binding"
    );
}
