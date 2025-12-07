//! Integration tests for plugin binding registration end-to-end
//!
//! These tests verify the complete workflow of plugin keybinding registration,
//! from building bindings through the plugin API to executing them in the editor.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use termide::editor::EditorMode;
use termide::input::input_handler::{InputHandler, MatchResult};
use termide::input::EditorCommand;
use termide_plugin_api::input::{PluginBindingBuilder, PluginInputExtension};

#[test]
fn test_plugin_binding_end_to_end_global() {
    // Create input handler
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register a global plugin binding through the plugin API
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "show_info")
        .global()
        .build()
        .expect("binding should build");

    input_handler
        .registry_mut()
        .register_keybinding(binding)
        .expect("plugin binding should register");

    // Test the binding works in Normal mode
    let key_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
    let result = input_handler.process_key_event(key_event, EditorMode::Normal);

    match result {
        MatchResult::Matched(cmd) => match cmd {
            EditorCommand::PluginCommand {
                plugin_name,
                command_name,
            } => {
                assert_eq!(plugin_name, "my-plugin");
                assert_eq!(command_name, "show_info");
            }
            _ => panic!("Expected PluginCommand, got {:?}", cmd),
        },
        _ => panic!("Expected Matched result, got {:?}", result),
    }

    // Test the binding works in Insert mode
    let result = input_handler.process_key_event(key_event, EditorMode::Insert);
    assert!(matches!(result, MatchResult::Matched(_)));

    // Test the binding does NOT work in Prompt mode (global excludes Prompt)
    let result = input_handler.process_key_event(key_event, EditorMode::Prompt);
    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_plugin_binding_end_to_end_mode_specific() {
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register a Normal-mode-only plugin binding
    let binding = PluginBindingBuilder::new("lsp")
        .bind("g d", "goto_definition")
        .in_mode("normal")
        .build()
        .expect("binding should build");

    input_handler
        .registry_mut()
        .register_keybinding(binding)
        .expect("plugin binding should register");

    // Test multi-key sequence in Normal mode
    let key_g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    let key_d = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);

    // First key - should be partial match
    let result = input_handler.process_key_event(key_g, EditorMode::Normal);
    assert!(matches!(result, MatchResult::Partial));

    // Second key - should match
    let result = input_handler.process_key_event(key_d, EditorMode::Normal);
    match result {
        MatchResult::Matched(cmd) => match cmd {
            EditorCommand::PluginCommand {
                plugin_name,
                command_name,
            } => {
                assert_eq!(plugin_name, "lsp");
                assert_eq!(command_name, "goto_definition");
            }
            _ => panic!("Expected PluginCommand"),
        },
        _ => panic!("Expected Matched result"),
    }

    // Test same sequence in Insert mode - should NOT match
    let result = input_handler.process_key_event(key_g, EditorMode::Insert);
    // Should not be partial in Insert mode
    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_plugin_binding_priority_over_defaults() {
    use termide::input::bindings::register_default_bindings;

    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register default bindings
    register_default_bindings(input_handler.registry_mut()).expect("defaults should register");

    // Register plugin binding for the same key as a default binding
    // Let's override 'i' in Normal mode (which normally switches to Insert mode)
    let binding = PluginBindingBuilder::new("vim-enhanced")
        .bind("i", "insert_before")
        .in_mode("normal")
        .build()
        .expect("binding should build");

    input_handler
        .registry_mut()
        .register_keybinding(binding)
        .expect("plugin binding should register");

    // Press 'i' in Normal mode - should trigger plugin command, not default
    let key_i = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    let result = input_handler.process_key_event(key_i, EditorMode::Normal);

    match result {
        MatchResult::Matched(cmd) => match cmd {
            EditorCommand::PluginCommand {
                plugin_name,
                command_name,
            } => {
                assert_eq!(plugin_name, "vim-enhanced");
                assert_eq!(command_name, "insert_before");
                // Plugin binding successfully overrode default binding
            }
            EditorCommand::ChangeMode(EditorMode::Insert) => {
                panic!("Default binding was used instead of plugin binding - priority system failed");
            }
            _ => panic!("Unexpected command"),
        },
        _ => panic!("Expected Matched result"),
    }
}

#[test]
fn test_multiple_plugins_coexist() {
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register bindings from multiple different plugins
    let lsp_binding = PluginBindingBuilder::new("lsp")
        .bind("g d", "goto_definition")
        .in_mode("normal")
        .build()
        .unwrap();

    let formatter_binding = PluginBindingBuilder::new("formatter")
        .bind("Ctrl+shift+f", "format_document")
        .global()
        .build()
        .unwrap();

    let autocomplete_binding = PluginBindingBuilder::new("autocomplete")
        .bind("Ctrl+space", "trigger")
        .in_mode("insert")
        .build()
        .unwrap();

    input_handler
        .registry_mut()
        .register_keybinding(lsp_binding)
        .unwrap();
    input_handler
        .registry_mut()
        .register_keybinding(formatter_binding)
        .unwrap();
    input_handler
        .registry_mut()
        .register_keybinding(autocomplete_binding)
        .unwrap();

    // Test LSP binding
    let key_g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    let key_d = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    input_handler.process_key_event(key_g, EditorMode::Normal);
    let result = input_handler.process_key_event(key_d, EditorMode::Normal);
    assert!(matches!(result, MatchResult::Matched(_)));

    // Test formatter binding (Ctrl+Shift+F)
    let key_f = KeyEvent::new(
        KeyCode::Char('f'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    let result = input_handler.process_key_event(key_f, EditorMode::Normal);
    assert!(matches!(result, MatchResult::Matched(_)));

    // Test autocomplete binding (Ctrl+Space in Insert mode)
    let key_space = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL);
    let result = input_handler.process_key_event(key_space, EditorMode::Insert);
    assert!(matches!(result, MatchResult::Matched(_)));

    // Autocomplete should NOT work in Normal mode
    let result = input_handler.process_key_event(key_space, EditorMode::Normal);
    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_plugin_binding_command_routing() {
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register bindings with different plugins
    let binding1 = PluginBindingBuilder::new("plugin-a")
        .bind("Ctrl+a", "cmd_a")
        .global()
        .build()
        .unwrap();

    let binding2 = PluginBindingBuilder::new("plugin-b")
        .bind("Ctrl+b", "cmd_b")
        .global()
        .build()
        .unwrap();

    input_handler
        .registry_mut()
        .register_keybinding(binding1)
        .unwrap();
    input_handler
        .registry_mut()
        .register_keybinding(binding2)
        .unwrap();

    // Test Ctrl+A routes to plugin-a
    let key_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let result = input_handler.process_key_event(key_a, EditorMode::Normal);

    match result {
        MatchResult::Matched(EditorCommand::PluginCommand { plugin_name, .. }) => {
            assert_eq!(plugin_name, "plugin-a");
        }
        _ => panic!("Expected plugin-a command"),
    }

    // Test Ctrl+B routes to plugin-b
    let key_b = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
    let result = input_handler.process_key_event(key_b, EditorMode::Normal);

    match result {
        MatchResult::Matched(EditorCommand::PluginCommand { plugin_name, .. }) => {
            assert_eq!(plugin_name, "plugin-b");
        }
        _ => panic!("Expected plugin-b command"),
    }
}
