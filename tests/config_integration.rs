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
    assert_eq!(result.unwrap().loaded, 1);

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
    assert_eq!(result.unwrap().loaded, 1);

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
    assert_eq!(result.unwrap().loaded, 1);

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
    assert_eq!(result.unwrap().loaded, 3);

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
    assert_eq!(result.unwrap().loaded, 1);

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
    assert_eq!(result.unwrap().loaded, 2); // Only Ctrl+S and Ctrl+K load successfully
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
    assert_eq!(result.unwrap().loaded, 1);

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

#[test]
fn test_empty_config_file_no_errors() {
    // Empty config file should not cause errors, defaults should still work
    let config_content = r#"
        # Empty configuration - just comments
        keybindings = []
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register defaults
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Load empty user config - should succeed with 0 bindings loaded
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 0);

    // Verify default bindings still work (Ctrl+S -> Save)
    let key_event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    match match_result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::Save);
        }
        _ => panic!("Default binding should still work with empty config"),
    }
}

#[test]
fn test_config_with_invalid_sequence() {
    // Config with invalid sequence should continue loading other bindings
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+"  # Invalid: incomplete modifier
        command = "quit"

        [[keybindings]]
        sequence = "Ctrl+k"
        command = "file.save"
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Should load 1 valid binding, skip the invalid one
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);

    // Verify the valid binding works
    let key_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    match match_result {
        MatchResult::Matched(cmd) => assert_eq!(cmd, EditorCommand::Save),
        _ => panic!("Valid binding should work"),
    }
}

#[test]
fn test_config_with_invalid_command() {
    // Config with invalid command should continue loading other bindings
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+x"
        command = "nonexistent_command"  # Invalid command

        [[keybindings]]
        sequence = "Ctrl+y"
        command = "quit"
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Should load 1 valid binding, skip the invalid one
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);

    // Verify the valid binding works
    let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    match match_result {
        MatchResult::Matched(cmd) => assert_eq!(cmd, EditorCommand::Quit),
        _ => panic!("Valid binding should work"),
    }
}

#[test]
fn test_config_with_invalid_mode() {
    // Config with invalid mode should continue loading other bindings
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+m"
        command = "quit"
        mode = "invalid_mode"  # Invalid mode

        [[keybindings]]
        sequence = "Ctrl+n"
        command = "file.save"
        mode = "normal"
    "#;

    let config_file = create_temp_config(config_content);
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Should load 1 valid binding, skip the invalid one
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);

    // Verify the valid binding works
    let key_event = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    match match_result {
        MatchResult::Matched(cmd) => assert_eq!(cmd, EditorCommand::Save),
        _ => panic!("Valid binding should work"),
    }
}

#[test]
fn test_user_binding_overrides_plugin_binding() {
    use termide_plugin_api::input::{PluginBindingBuilder, PluginInputExtension};

    // Test priority system: User > Plugin > Default
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register default bindings
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Register a plugin binding for Ctrl+P
    let plugin_binding = PluginBindingBuilder::new("test-plugin")
        .bind("Ctrl+p", "plugin_action")
        .global()
        .build()
        .unwrap();

    input_handler
        .registry_mut()
        .register_keybinding(plugin_binding)
        .unwrap();

    // Verify plugin binding works
    let key_event = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    match match_result {
        MatchResult::Matched(cmd) => match cmd {
            EditorCommand::PluginCommand { plugin_name, .. } => {
                assert_eq!(plugin_name, "test-plugin");
            }
            _ => panic!("Expected plugin command"),
        },
        _ => panic!("Plugin binding should work"),
    }

    // Now load user config that overrides the same key
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+p"
        command = "quit"
    "#;

    let config_file = create_temp_config(config_content);
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);

    // User binding should now override plugin binding
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    match match_result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::Quit);
            // Success! User binding overrode plugin binding
        }
        _ => panic!("User binding should override plugin binding"),
    }
}

#[test]
fn test_plugin_then_config_user_wins() {
    use termide_plugin_api::input::{PluginBindingBuilder, PluginInputExtension};

    // Plugin binding registered first, then config loads - user config should win
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register default bindings
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Register plugin binding for Ctrl+T
    let plugin_binding = PluginBindingBuilder::new("formatter")
        .bind("Ctrl+t", "format")
        .in_mode("normal")
        .build()
        .unwrap();

    input_handler
        .registry_mut()
        .register_keybinding(plugin_binding)
        .unwrap();

    // Load user config with different command for same key
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+t"
        command = "delete_char"
        mode = "normal"
    "#;

    let config_file = create_temp_config(config_content);
    let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);

    // Process Ctrl+T in Normal mode
    let key_event = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    // Should match user binding (DeleteChar), not plugin binding
    match match_result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::DeleteChar);
            // User config wins over plugin binding
        }
        _ => panic!("User config should override plugin binding"),
    }
}

#[test]
fn test_all_three_priority_levels() {
    use termide_plugin_api::input::{PluginBindingBuilder, PluginInputExtension};

    // Test all three priority levels: User > Plugin > Default
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register defaults
    register_default_bindings(&mut input_handler.registry_mut()).unwrap();

    // Default binding: Ctrl+S -> Save (from defaults)
    // Plugin binding: Ctrl+S -> plugin action
    // User binding: Ctrl+S -> Quit

    // Add plugin binding
    let plugin_binding = PluginBindingBuilder::new("override-test")
        .bind("Ctrl+s", "plugin_save")
        .global()
        .build()
        .unwrap();

    input_handler
        .registry_mut()
        .register_keybinding(plugin_binding)
        .unwrap();

    // At this point, plugin binding should override default
    let key_event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    match match_result {
        MatchResult::Matched(EditorCommand::PluginCommand { plugin_name, .. }) => {
            assert_eq!(plugin_name, "override-test");
        }
        _ => panic!("Plugin should override default"),
    }

    // Now add user config
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+s"
        command = "quit"
    "#;

    let config_file = create_temp_config(config_content);
    load_user_keybindings(&mut input_handler.registry_mut(), config_file.path()).unwrap();

    // User binding should override both plugin and default
    let match_result = input_handler.process_key_event(key_event, EditorMode::Normal);

    match match_result {
        MatchResult::Matched(cmd) => {
            assert_eq!(cmd, EditorCommand::Quit);
            // User binding wins!
        }
        _ => panic!("User binding should override plugin and default"),
    }
}
