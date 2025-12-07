//! Integration tests for config file hot reload
//!
//! These tests verify that the hot reload feature works end-to-end:
//! - ConfigWatcher detects file changes
//! - reload_user_keybindings clears old bindings and loads new ones
//! - Priority system remains intact after reload

use std::fs;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;
use termide::input::config::reload_user_keybindings;
use termide::input::keybinding::{KeyPattern, Priority};
use termide::input::registry::KeyBindingRegistry;
use termide::input::watcher::ConfigWatcher;
use crossterm::event::{KeyCode, KeyModifiers};

/// Helper function to create a temporary config file with given content
fn create_temp_config(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes()).expect("Failed to write config");
    file.flush().expect("Failed to flush config");
    file
}

/// Helper function to update a config file with new content
fn update_config_file(path: &Path, content: &str) {
    fs::write(path, content).expect("Failed to update config file");
    // Give the filesystem a moment to process the write
    thread::sleep(Duration::from_millis(50));
}

#[test]
fn test_watcher_detects_file_modification() {
    let config_content = r#"
[[keybindings]]
sequence = "Ctrl+s"
command = "file.save"
"#;

    let temp_file = create_temp_config(config_content);
    let config_path = temp_file.path();

    // Create watcher
    let mut watcher = ConfigWatcher::new(config_path).expect("Failed to create watcher");

    // Initially, no changes
    assert!(!watcher.check_for_changes());

    // Modify the file
    let new_content = r#"
[[keybindings]]
sequence = "Ctrl+q"
command = "file.quit"
"#;
    update_config_file(config_path, new_content);

    // Wait for debouncer to process events (500ms debounce + buffer)
    thread::sleep(Duration::from_millis(700));

    // Should detect changes
    assert!(watcher.check_for_changes(), "Watcher should detect file modification");

    // Second check should return false (no new changes)
    assert!(!watcher.check_for_changes(), "No new changes after first check");
}

#[test]
fn test_reload_removes_old_bindings_and_loads_new() {
    let initial_config = r#"
[[keybindings]]
sequence = "Ctrl+s"
command = "file.save"

[[keybindings]]
sequence = "Ctrl+q"
command = "file.quit"
"#;

    let temp_file = create_temp_config(initial_config);
    let config_path = temp_file.path();

    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Load initial config
    let loaded = termide::input::config::load_user_keybindings(&mut registry, config_path)
        .expect("Failed to load initial config");
    assert_eq!(loaded, 2, "Should load 2 bindings initially");

    // Modify config - remove one binding, add a new one
    let new_config = r#"
[[keybindings]]
sequence = "Ctrl+s"
command = "file.save"

[[keybindings]]
sequence = "Ctrl+n"
command = "file.new"
"#;
    update_config_file(config_path, new_config);

    // Reload config
    let (removed, loaded) = reload_user_keybindings(&mut registry, config_path)
        .expect("Failed to reload config");

    assert_eq!(removed, 2, "Should remove 2 old bindings");
    assert_eq!(loaded, 2, "Should load 2 new bindings");

    // Verify old Ctrl+q binding is gone
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('q'), KeyModifiers::CONTROL));
    let cmd = registry.find_match(termide::editor::EditorMode::Normal);
    assert!(cmd.is_none(), "Old Ctrl+q binding should be removed");

    // Verify new Ctrl+n binding exists
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('n'), KeyModifiers::CONTROL));
    let cmd = registry.find_match(termide::editor::EditorMode::Normal);
    assert!(cmd.is_some(), "New Ctrl+n binding should exist");
}

#[test]
fn test_reload_preserves_default_and_plugin_bindings() {
    let config_content = r#"
[[keybindings]]
sequence = "Ctrl+s"
command = "file.save"
"#;

    let temp_file = create_temp_config(config_content);
    let config_path = temp_file.path();

    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register a default binding
    let default_binding = termide::input::keybinding::KeyBinding::new(
        termide::input::keybinding::KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
        ]).unwrap(),
        termide::input::EditorCommand::ChangeMode(termide::editor::EditorMode::Insert),
        termide::input::keybinding::BindingContext::Mode(termide::editor::EditorMode::Normal),
        Priority::Default,
    );
    registry.register(default_binding).unwrap();

    // Register a plugin binding
    let plugin_binding = termide::input::keybinding::KeyBinding::new(
        termide::input::keybinding::KeySequence::new(vec![
            KeyPattern::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
        ]).unwrap(),
        termide::input::EditorCommand::PluginCommand {
            plugin_name: "test_plugin".to_string(),
            command_name: "test_cmd".to_string(),
        },
        termide::input::keybinding::BindingContext::Global,
        Priority::Plugin,
    );
    registry.register(plugin_binding).unwrap();

    // Load user config
    termide::input::config::load_user_keybindings(&mut registry, config_path)
        .expect("Failed to load config");

    assert_eq!(registry.len(), 3, "Should have default + plugin + user bindings");

    // Reload user config (empty this time)
    let empty_config = "";
    update_config_file(config_path, empty_config);

    let (removed, loaded) = reload_user_keybindings(&mut registry, config_path)
        .expect("Failed to reload config");

    assert_eq!(removed, 1, "Should remove 1 user binding");
    assert_eq!(loaded, 0, "Should load 0 new bindings");
    assert_eq!(registry.len(), 2, "Should still have default + plugin bindings");

    // Verify default binding still works
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE));
    let cmd = registry.find_match(termide::editor::EditorMode::Normal);
    assert!(cmd.is_some(), "Default binding should still exist");

    // Verify plugin binding still works
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('p'), KeyModifiers::CONTROL));
    let cmd = registry.find_match(termide::editor::EditorMode::Normal);
    assert!(cmd.is_some(), "Plugin binding should still exist");
}

#[test]
fn test_reload_with_config_errors_clears_old_bindings() {
    let initial_config = r#"
[[keybindings]]
sequence = "Ctrl+s"
command = "file.save"
"#;

    let temp_file = create_temp_config(initial_config);
    let config_path = temp_file.path();

    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Load initial config
    termide::input::config::load_user_keybindings(&mut registry, config_path)
        .expect("Failed to load initial config");
    assert_eq!(registry.len(), 1);

    // Update config with invalid TOML
    let invalid_config = "this is not valid TOML {[}";
    update_config_file(config_path, invalid_config);

    // Reload should fail but still clear old bindings
    let result = reload_user_keybindings(&mut registry, config_path);
    assert!(result.is_err(), "Reload should fail with invalid config");
    assert_eq!(registry.len(), 0, "Old bindings should be cleared even if reload fails");
}

#[test]
fn test_reload_multiple_times() {
    let temp_file = create_temp_config("");
    let config_path = temp_file.path();

    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // First reload - add one binding
    let config1 = r#"
[[keybindings]]
sequence = "Ctrl+a"
command = "file.select_all"
"#;
    update_config_file(config_path, config1);
    let (_, loaded) = reload_user_keybindings(&mut registry, config_path).unwrap();
    assert_eq!(loaded, 1);
    assert_eq!(registry.len(), 1);

    // Second reload - add another binding
    let config2 = r#"
[[keybindings]]
sequence = "Ctrl+a"
command = "file.select_all"

[[keybindings]]
sequence = "Ctrl+b"
command = "file.bold"
"#;
    update_config_file(config_path, config2);
    let (removed, loaded) = reload_user_keybindings(&mut registry, config_path).unwrap();
    assert_eq!(removed, 1);
    assert_eq!(loaded, 2);
    assert_eq!(registry.len(), 2);

    // Third reload - remove all bindings
    let config3 = "";
    update_config_file(config_path, config3);
    let (removed, loaded) = reload_user_keybindings(&mut registry, config_path).unwrap();
    assert_eq!(removed, 2);
    assert_eq!(loaded, 0);
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_reload_with_mode_specific_bindings() {
    let config_content = r#"
[[keybindings]]
sequence = "j"
command = "cursor.down"
mode = "normal"

[[keybindings]]
sequence = "k"
command = "cursor.up"
mode = "normal"

[[keybindings]]
sequence = "Ctrl+a"
command = "file.select_all"
# No mode = global binding
"#;

    let temp_file = create_temp_config(config_content);
    let config_path = temp_file.path();

    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Load initial config
    termide::input::config::load_user_keybindings(&mut registry, config_path)
        .expect("Failed to load config");
    assert_eq!(registry.len(), 3);

    // Reload with different mode-specific bindings
    let new_config = r#"
[[keybindings]]
sequence = "j"
command = "cursor.down"
mode = "insert"

[[keybindings]]
sequence = "Ctrl+a"
command = "file.select_all"
"#;
    update_config_file(config_path, new_config);

    let (removed, loaded) = reload_user_keybindings(&mut registry, config_path)
        .expect("Failed to reload");

    assert_eq!(removed, 3, "Should remove all old bindings");
    assert_eq!(loaded, 2, "Should load 2 new bindings");
    assert_eq!(registry.len(), 2);
}

#[test]
fn test_watcher_and_reload_end_to_end() {
    let initial_config = r#"
[[keybindings]]
sequence = "Ctrl+s"
command = "file.save"
"#;

    let temp_file = create_temp_config(initial_config);
    let config_path = temp_file.path();

    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    let mut watcher = ConfigWatcher::new(config_path).expect("Failed to create watcher");

    // Load initial config
    termide::input::config::load_user_keybindings(&mut registry, config_path)
        .expect("Failed to load initial config");
    assert_eq!(registry.len(), 1);

    // Modify config
    let new_config = r#"
[[keybindings]]
sequence = "Ctrl+q"
command = "file.quit"

[[keybindings]]
sequence = "Ctrl+n"
command = "file.new"
"#;
    update_config_file(config_path, new_config);

    // Wait for watcher to detect changes
    thread::sleep(Duration::from_millis(700));

    // Check for changes
    if watcher.check_for_changes() {
        // Reload config
        let (removed, loaded) = reload_user_keybindings(&mut registry, config_path)
            .expect("Failed to reload config");

        assert_eq!(removed, 1, "Should remove 1 old binding");
        assert_eq!(loaded, 2, "Should load 2 new bindings");
        assert_eq!(registry.len(), 2);
    } else {
        panic!("Watcher should have detected changes");
    }
}
