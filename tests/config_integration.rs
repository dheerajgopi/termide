//! Integration tests for user configuration loading
//!
//! These tests verify that the config loading system works end-to-end,
//! including priority ordering, override behavior, and mode-specific bindings.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;
use termide::editor::EditorMode;
use termide::input::bindings::register_default_bindings;
use termide::input::config::load_user_keybindings;
use termide::input::input_handler::{InputHandler, MatchResult};
use termide::input::EditorCommand;

/// Helper to create a temporary config file with content
fn create_temp_config(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

#[test]
fn test_user_binding_overrides_default() {
    // User config overrides Ctrl+s from Save to Quit
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+s"
        command = "quit"
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register defaults (Ctrl+S -> Save)
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Load user config (Ctrl+s -> Quit)
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);

    // Process Ctrl+S key event
    let key_event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    // Should match user binding (Quit) instead of default (Save)
    match match_result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::Quit);
        }
        _ => panic!("Expected Matched result, got {:?}", match_result),
    }
}

#[test]
fn test_user_binding_adds_new_binding() {
    // User adds a new binding not in defaults
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+k"
        command = "quit"
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register defaults (no Ctrl+K binding)
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Load user config (Ctrl+K -> Quit)
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);

    // Process Ctrl+K key event
    let key_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    // Should match user binding
    match match_result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::Quit);
        }
        _ => panic!("Expected Matched result, got {:?}", match_result),
    }
}

#[test]
fn test_mode_specific_user_binding() {
    // User adds mode-specific binding
    let config_content = r#"
        [[keybindings]]
        sequence = "x"
        command = "delete_char"
        mode = "normal"
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register defaults
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Load user config
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);

    // Process 'x' in Normal mode - should match
    let key_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    match match_result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::DeleteChar);
        }
        _ => panic!("Expected Matched result in Normal mode, got {:?}", match_result),
    }

    // Process 'x' in Insert mode - should NOT match (mode-specific binding)
    let match_result_insert = input_handler.process_key_event(key_event, EditorMode::Insert);
    match match_result_insert {
        MatchResult::NoMatch => {
            // Expected - binding is specific to Normal mode
        }
        other => panic!("Expected NoMatch in Insert mode, got {:?}", other),
    }
}

#[test]
fn test_multiple_user_bindings() {
    // User config with multiple bindings
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+s"
        command = "quit"

        [[keybindings]]
        sequence = "Ctrl+q"
        command = "file.save"

        [[keybindings]]
        sequence = "d d"
        command = "delete_char"
        mode = "normal"
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register defaults
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Load user config - should load all 3 bindings
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);

    // Test Ctrl+S -> Quit
    let key_event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);
    match match_result {
        MatchResult::Matched(cmd) => assert_eq!(cmd, EditorCommand::Quit),
        _ => panic!("Expected Quit command"),
    }

    // Test Ctrl+Q -> Save
    let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);
    match match_result {
        MatchResult::Matched(cmd) => assert_eq!(cmd, EditorCommand::Save),
        _ => panic!("Expected Save command"),
    }

    // Test d d -> DeleteChar (multi-key sequence)
    let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    let match_result1 = input_handler.process_key_event(key_event, EditorMode::Normal);
    assert!(matches!(match_result1, MatchResult::Partial));

    let match_result2 = input_handler.process_key_event(key_event, EditorMode::Normal);
    match match_result2 {
        MatchResult::Matched(cmd) => assert_eq!(cmd, EditorCommand::DeleteChar),
        _ => panic!("Expected DeleteChar command for 'd d' sequence"),
    }
}

#[test]
fn test_plugin_command_in_config() {
    // User config with plugin command
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+f"
        command = "rust_analyzer.format"
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register defaults
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Load user config
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);

    // Process Ctrl+F key event
    let key_event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    // Should match plugin command
    match match_result {
        MatchResult::Matched(cmd) => {
            assert_eq!(
                cmd,
                EditorCommand::PluginCommand {
                    plugin_name: "rust_analyzer".to_string(),
                    command_name: "format".to_string()
                }
            );
        }
        _ => panic!("Expected Matched result with plugin command, got {:?}", match_result),
    }
}

#[test]
fn test_mixed_valid_invalid_bindings() {
    // Config with mix of valid and invalid bindings
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+s"
        command = "quit"

        [[keybindings]]
        sequence = "Ctr+Q"  # Invalid: typo in modifier
        command = "file.save"

        [[keybindings]]
        sequence = "Ctrl+k"
        command = "delete_char"

        [[keybindings]]
        sequence = "Ctrl+x"
        command = "unknown_cmd"  # Invalid: unknown command
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register defaults
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Load user config - should load 2 valid bindings, skip 2 invalid
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2); // Only Ctrl+S and Ctrl+K load successfully
}

#[test]
fn test_global_binding_works_in_all_modes() {
    // User config with global binding (no mode specified)
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+g"
        command = "quit"
        # No mode specified - should work in all modes
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register defaults
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Load user config
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);

    let key_event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);

    // Test in Normal mode
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);
    match match_result {
        MatchResult::Matched(cmd) => assert_eq!(cmd, EditorCommand::Quit),
        _ => panic!("Expected Quit in Normal mode"),
    }

    // Test in Insert mode
    let match_result = input_handler.process_key_event(key_event, EditorMode::Insert);
    match match_result {
        MatchResult::Matched(cmd) => assert_eq!(cmd, EditorCommand::Quit),
        _ => panic!("Expected Quit in Insert mode"),
    }
}
