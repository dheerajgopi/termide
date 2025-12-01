//! Unit tests for default keybindings

use crate::editor::EditorMode;
use crate::input::bindings::{
    arrow_key_navigation, global_bindings, insert_mode_bindings, normal_mode_bindings,
    prompt_mode_bindings, register_default_bindings,
};
use crate::input::keybinding::{BindingContext, Priority};
use crate::input::registry::KeyBindingRegistry;
use crate::input::{Direction, EditorCommand};
use std::time::Duration;

#[test]
fn test_register_default_bindings_no_conflicts() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Registering default bindings should not have any conflicts
    let result = register_default_bindings(&mut registry);
    assert!(result.is_ok(), "Default bindings should register without conflicts");
    assert!(!registry.is_empty(), "Registry should contain bindings");
}

#[test]
fn test_global_bindings_active_in_correct_modes() {
    let bindings = global_bindings();

    for binding in bindings {
        assert_eq!(binding.context(), &BindingContext::Global);
        assert_eq!(binding.priority(), Priority::Default);
        // Global bindings should be active in Insert and Normal, not in Prompt
        assert!(binding.context().is_active(EditorMode::Insert));
        assert!(binding.context().is_active(EditorMode::Normal));
        assert!(!binding.context().is_active(EditorMode::Prompt));
    }
}

#[test]
fn test_global_save_and_quit_bindings_exist() {
    let bindings = global_bindings();

    let has_save = bindings.iter().any(|b| matches!(b.command(), EditorCommand::Save));
    let has_quit = bindings.iter().any(|b| matches!(b.command(), EditorCommand::Quit));

    assert!(has_save, "Save binding should exist");
    assert!(has_quit, "Quit binding should exist");
}

#[test]
fn test_insert_mode_essential_bindings_exist() {
    let bindings = insert_mode_bindings();

    let has_newline = bindings.iter().any(|b| matches!(b.command(), EditorCommand::InsertChar('\n')));
    let has_delete = bindings.iter().any(|b| matches!(b.command(), EditorCommand::DeleteChar));
    let has_mode_change = bindings.iter().any(|b| matches!(b.command(), EditorCommand::ChangeMode(EditorMode::Normal)));

    assert!(has_newline, "Enter binding should exist");
    assert!(has_delete, "Backspace binding should exist");
    assert!(has_mode_change, "Escape to Normal mode binding should exist");
}

#[test]
fn test_normal_mode_insert_binding_exists() {
    let bindings = normal_mode_bindings();

    let has_insert_mode = bindings
        .iter()
        .any(|b| matches!(b.command(), EditorCommand::ChangeMode(EditorMode::Insert)));

    assert!(has_insert_mode, "'i' to Insert mode binding should exist");
}

#[test]
fn test_normal_mode_i_only_active_in_normal() {
    let bindings = normal_mode_bindings();

    let i_binding = bindings
        .iter()
        .find(|b| matches!(b.command(), EditorCommand::ChangeMode(EditorMode::Insert)))
        .expect("'i' binding should exist");

    assert!(i_binding.context().is_active(EditorMode::Normal));
    assert!(!i_binding.context().is_active(EditorMode::Insert));
    assert!(!i_binding.context().is_active(EditorMode::Prompt));
}

#[test]
fn test_prompt_mode_essential_bindings_exist() {
    let bindings = prompt_mode_bindings();

    let has_delete = bindings.iter().any(|b| matches!(b.command(), EditorCommand::PromptDeleteChar));
    let has_accept = bindings.iter().any(|b| matches!(b.command(), EditorCommand::AcceptPrompt));
    let has_cancel = bindings.iter().any(|b| matches!(b.command(), EditorCommand::CancelPrompt));

    assert!(has_delete, "Prompt backspace binding should exist");
    assert!(has_accept, "Prompt accept binding should exist");
    assert!(has_cancel, "Prompt cancel binding should exist");
}

#[test]
fn test_prompt_bindings_only_active_in_prompt() {
    let bindings = prompt_mode_bindings();

    for binding in bindings {
        assert!(!binding.context().is_active(EditorMode::Insert));
        assert!(!binding.context().is_active(EditorMode::Normal));
        assert!(binding.context().is_active(EditorMode::Prompt));
    }
}

#[test]
fn test_arrow_key_navigation_all_directions_exist() {
    let bindings = arrow_key_navigation(vec![EditorMode::Insert]);

    let has_up = bindings.iter().any(|b| matches!(b.command(), EditorCommand::MoveCursor(Direction::Up)));
    let has_down = bindings.iter().any(|b| matches!(b.command(), EditorCommand::MoveCursor(Direction::Down)));
    let has_left = bindings.iter().any(|b| matches!(b.command(), EditorCommand::MoveCursor(Direction::Left)));
    let has_right = bindings.iter().any(|b| matches!(b.command(), EditorCommand::MoveCursor(Direction::Right)));

    assert!(has_up, "Up arrow binding should exist");
    assert!(has_down, "Down arrow binding should exist");
    assert!(has_left, "Left arrow binding should exist");
    assert!(has_right, "Right arrow binding should exist");
}

#[test]
fn test_arrow_keys_use_modes_context_for_reuse() {
    let modes = vec![EditorMode::Insert, EditorMode::Normal];
    let bindings = arrow_key_navigation(modes.clone());

    for binding in bindings {
        assert!(matches!(
            binding.context(),
            BindingContext::Modes(ref m) if m == &modes
        ), "Arrow keys should use Modes context for reuse across multiple modes");
    }
}

#[test]
fn test_arrow_keys_no_duplication_in_insert_bindings() {
    // Arrow keys should be defined once and reused via Modes context
    let insert_bindings = insert_mode_bindings();

    // All arrow key bindings should use Modes context, not Mode(Insert)
    for binding in insert_bindings.iter().filter(|b| matches!(b.command(), EditorCommand::MoveCursor(_))) {
        assert!(
            matches!(binding.context(), BindingContext::Modes(_)),
            "Arrow keys should use Modes context to avoid duplication"
        );
    }
}

#[test]
fn test_arrow_keys_active_in_insert_and_normal() {
    let bindings = insert_mode_bindings();

    let arrow_binding = bindings
        .iter()
        .find(|b| matches!(b.command(), EditorCommand::MoveCursor(_)))
        .expect("Arrow key binding should exist");

    // Should be active in both Insert and Normal modes
    assert!(arrow_binding.context().is_active(EditorMode::Insert));
    assert!(arrow_binding.context().is_active(EditorMode::Normal));
    assert!(!arrow_binding.context().is_active(EditorMode::Prompt));
}

#[test]
fn test_all_default_bindings_use_default_priority() {
    let mut all_bindings = Vec::new();
    all_bindings.extend(global_bindings());
    all_bindings.extend(insert_mode_bindings());
    all_bindings.extend(normal_mode_bindings());
    all_bindings.extend(prompt_mode_bindings());

    for binding in all_bindings {
        assert_eq!(binding.priority(), Priority::Default, "All default bindings should have Priority::Default");
    }
}
