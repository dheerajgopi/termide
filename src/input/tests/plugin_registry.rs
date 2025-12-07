//! Unit tests for plugin binding registration
//!
//! Tests the PluginInputExtension trait implementation for KeyBindingRegistry

use crate::editor::EditorMode;
use crate::input::keybinding::KeyPattern;
use crate::input::registry::KeyBindingRegistry;
use crate::input::EditorCommand;
use crossterm::event::{KeyCode, KeyModifiers};
use std::time::Duration;
use termide_plugin_api::input::{PluginBindingBuilder, PluginInputExtension};

// ============================================================================
// Successful Plugin Binding Registration Tests
// ============================================================================

#[test]
fn test_register_plugin_binding_global() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Build a global plugin binding
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "show_info")
        .global()
        .build()
        .expect("binding should build");

    // Register the binding
    let result = registry.register_keybinding(binding);
    assert!(result.is_ok(), "global plugin binding should register successfully");

    // Verify the binding was registered
    assert_eq!(registry.len(), 1);

    // Verify it works in Normal mode (global context includes Normal)
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
    let cmd = registry.find_match(EditorMode::Normal);
    assert!(cmd.is_some(), "should match in Normal mode");

    match cmd.unwrap() {
        EditorCommand::PluginCommand { plugin_name, command_name } => {
            assert_eq!(plugin_name, "my-plugin");
            assert_eq!(command_name, "show_info");
        }
        _ => panic!("Expected PluginCommand"),
    }

    // Clear and test Insert mode
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
    assert!(registry.find_match(EditorMode::Insert).is_some(), "should match in Insert mode");

    // Clear and test Prompt mode (global excludes Prompt)
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
    assert!(registry.find_match(EditorMode::Prompt).is_none(), "should NOT match in Prompt mode");
}

#[test]
fn test_register_plugin_binding_single_mode() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Build a Normal-mode-only binding
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("g d", "goto_definition")
        .in_mode("normal")
        .build()
        .expect("binding should build");

    // Register the binding
    let result = registry.register_keybinding(binding);
    assert!(result.is_ok(), "mode-specific plugin binding should register successfully");

    assert_eq!(registry.len(), 1);

    // Test in Normal mode - should match
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE));
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    let cmd = registry.find_match(EditorMode::Normal);
    assert!(cmd.is_some(), "should match in Normal mode");

    // Clear and test in Insert mode - should NOT match
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE));
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    assert!(registry.find_match(EditorMode::Insert).is_none(), "should NOT match in Insert mode");
}

#[test]
fn test_register_plugin_binding_multiple_modes() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Build a multi-mode binding (Insert and Normal)
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+/", "toggle_comment")
        .in_modes(&["insert", "normal"])
        .build()
        .expect("binding should build");

    // Register the binding
    let result = registry.register_keybinding(binding);
    assert!(result.is_ok(), "multi-mode plugin binding should register successfully");

    // Test in Insert mode
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('/'), KeyModifiers::CONTROL));
    assert!(registry.find_match(EditorMode::Insert).is_some(), "should match in Insert mode");

    // Test in Normal mode
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('/'), KeyModifiers::CONTROL));
    assert!(registry.find_match(EditorMode::Normal).is_some(), "should match in Normal mode");

    // Test in Prompt mode - should NOT match
    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('/'), KeyModifiers::CONTROL));
    assert!(registry.find_match(EditorMode::Prompt).is_none(), "should NOT match in Prompt mode");
}

#[test]
fn test_register_plugin_binding_multi_key_sequence() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Build a multi-key sequence binding
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("g g", "goto_top")
        .in_mode("normal")
        .build()
        .expect("binding should build");

    let result = registry.register_keybinding(binding);
    assert!(result.is_ok(), "multi-key plugin binding should register successfully");

    // Add first 'g' - partial match
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE));
    assert!(registry.is_partial_match(EditorMode::Normal), "should be partial match");
    assert!(registry.find_match(EditorMode::Normal).is_none(), "should not have complete match yet");

    // Add second 'g' - complete match
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE));
    let cmd = registry.find_match(EditorMode::Normal);
    assert!(cmd.is_some(), "should have complete match");
    assert!(!registry.is_partial_match(EditorMode::Normal), "should not be partial anymore");
}

#[test]
fn test_register_multiple_plugin_bindings() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register multiple bindings from same plugin
    let binding1 = PluginBindingBuilder::new("lsp")
        .bind("g d", "goto_definition")
        .in_mode("normal")
        .build()
        .unwrap();

    let binding2 = PluginBindingBuilder::new("lsp")
        .bind("g r", "find_references")
        .in_mode("normal")
        .build()
        .unwrap();

    let binding3 = PluginBindingBuilder::new("lsp")
        .bind("h", "hover")
        .in_mode("normal")
        .build()
        .unwrap();

    registry.register_keybinding(binding1).unwrap();
    registry.register_keybinding(binding2).unwrap();
    registry.register_keybinding(binding3).unwrap();

    assert_eq!(registry.len(), 3);

    // Test each binding works
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE));
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE));
    assert!(registry.find_match(EditorMode::Normal).is_some());

    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE));
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('r'), KeyModifiers::NONE));
    assert!(registry.find_match(EditorMode::Normal).is_some());

    registry.clear_sequence();
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('h'), KeyModifiers::NONE));
    assert!(registry.find_match(EditorMode::Normal).is_some());
}

#[test]
fn test_register_bindings_from_different_plugins() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let binding1 = PluginBindingBuilder::new("rust-analyzer")
        .bind("Ctrl+shift+F", "format")
        .global()
        .build()
        .unwrap();

    let binding2 = PluginBindingBuilder::new("autocomplete")
        .bind("Ctrl+space", "trigger")
        .in_mode("insert")
        .build()
        .unwrap();

    registry.register_keybinding(binding1).unwrap();
    registry.register_keybinding(binding2).unwrap();

    assert_eq!(registry.len(), 2);
}

#[test]
fn test_plugin_binding_auto_namespacing() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Builder should auto-namespace the command
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "show_info")  // Just "show_info", not "my-plugin.show_info"
        .global()
        .build()
        .unwrap();

    registry.register_keybinding(binding).unwrap();

    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
    let cmd = registry.find_match(EditorMode::Normal);

    match cmd.unwrap() {
        EditorCommand::PluginCommand { plugin_name, command_name } => {
            assert_eq!(plugin_name, "my-plugin", "plugin name should be extracted");
            assert_eq!(command_name, "show_info", "command name should be extracted");
        }
        _ => panic!("Expected PluginCommand"),
    }
}

// ============================================================================
// Plugin Binding Conflict Detection Tests
// ============================================================================

#[test]
fn test_plugin_binding_conflict_same_plugin() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register first binding
    let binding1 = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "show_info")
        .global()
        .build()
        .unwrap();

    registry.register_keybinding(binding1).unwrap();

    // Try to register same sequence+context from same plugin - should conflict
    let binding2 = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "different_command")
        .global()
        .build()
        .unwrap();

    let result = registry.register_keybinding(binding2);
    assert!(result.is_err(), "should detect conflict");

    match result {
        Err(termide_plugin_api::input::BindingError::Conflict { sequence, .. }) => {
            assert_eq!(sequence, "Ctrl+k");
        }
        _ => panic!("Expected Conflict error"),
    }
}

#[test]
fn test_plugin_binding_conflict_different_plugins() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register binding from first plugin
    let binding1 = PluginBindingBuilder::new("plugin-a")
        .bind("Ctrl+k", "cmd_a")
        .global()
        .build()
        .unwrap();

    registry.register_keybinding(binding1).unwrap();

    // Try to register same sequence+context from different plugin - should also conflict
    // Both are Plugin priority, so they conflict
    let binding2 = PluginBindingBuilder::new("plugin-b")
        .bind("Ctrl+k", "cmd_b")
        .global()
        .build()
        .unwrap();

    let result = registry.register_keybinding(binding2);
    assert!(result.is_err(), "different plugins with same sequence should conflict");
}

#[test]
fn test_plugin_binding_no_conflict_different_context() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register Ctrl+k in Normal mode
    let binding1 = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "cmd_normal")
        .in_mode("normal")
        .build()
        .unwrap();

    registry.register_keybinding(binding1).unwrap();

    // Register same sequence in Insert mode - should NOT conflict (different context)
    let binding2 = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "cmd_insert")
        .in_mode("insert")
        .build()
        .unwrap();

    let result = registry.register_keybinding(binding2);
    assert!(result.is_ok(), "same sequence in different contexts should not conflict");
    assert_eq!(registry.len(), 2);
}

#[test]
fn test_plugin_binding_no_conflict_different_sequence() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register Ctrl+k
    let binding1 = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "cmd_k")
        .global()
        .build()
        .unwrap();

    registry.register_keybinding(binding1).unwrap();

    // Register Ctrl+j - should NOT conflict (different sequence)
    let binding2 = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+j", "cmd_j")
        .global()
        .build()
        .unwrap();

    let result = registry.register_keybinding(binding2);
    assert!(result.is_ok(), "different sequences should not conflict");
    assert_eq!(registry.len(), 2);
}

// ============================================================================
// Parse Error Handling Tests
// ============================================================================

#[test]
fn test_plugin_binding_invalid_sequence_format() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Invalid key sequence (unknown modifier)
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctr+K", "show_info")  // Typo: "Ctr" instead of "Ctrl"
        .global()
        .build()
        .unwrap();

    let result = registry.register_keybinding(binding);
    assert!(result.is_err(), "should reject invalid key sequence");

    match result {
        Err(termide_plugin_api::input::BindingError::InvalidSequence(seq, reason)) => {
            assert_eq!(seq, "Ctr+K");
            assert!(reason.contains("modifier") || reason.contains("unknown"), "error should mention invalid modifier");
        }
        _ => panic!("Expected InvalidSequence error"),
    }
}

#[test]
fn test_plugin_binding_invalid_key_name() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Invalid key name
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+InvalidKey", "show_info")
        .global()
        .build()
        .unwrap();

    let result = registry.register_keybinding(binding);
    assert!(result.is_err(), "should reject invalid key name");
}

#[test]
fn test_plugin_binding_empty_sequence() {
    // The builder itself should prevent empty sequences
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("", "show_info")  // Empty sequence
        .global()
        .build();

    // Builder validation should catch this
    assert!(binding.is_err(), "builder should reject empty sequence");
}

#[test]
fn test_plugin_binding_empty_command() {
    // The builder itself should prevent empty commands
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "")  // Empty command
        .global()
        .build();

    // Builder validation should catch this
    assert!(binding.is_err(), "builder should reject empty command");
}

#[test]
fn test_plugin_binding_validates_plugin_command_format() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // The builder auto-namespaces, so we'll get "my-plugin.show_info"
    // This should parse correctly as a PluginCommand
    let binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "show_info")
        .global()
        .build()
        .unwrap();

    let result = registry.register_keybinding(binding);
    assert!(result.is_ok(), "properly namespaced plugin command should work");
}

// ============================================================================
// Priority System Integration Tests
// ============================================================================

#[test]
fn test_user_binding_overrides_plugin_binding() {
    use crate::input::keybinding::{KeyBinding, KeySequence, KeyPattern, BindingContext, Priority};
    use std::str::FromStr;

    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register plugin binding for Ctrl+s
    let plugin_binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+s", "plugin_save")
        .global()
        .build()
        .unwrap();

    registry.register_keybinding(plugin_binding).unwrap();

    // Register user binding for same key with User priority
    let user_binding = KeyBinding::new(
        KeySequence::from_str("Ctrl+s").unwrap(),
        EditorCommand::Save,
        BindingContext::Global,
        Priority::User,
    );

    registry.register(user_binding).unwrap();

    assert_eq!(registry.len(), 2, "both bindings should be registered");

    // Add Ctrl+s to buffer
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('s'), KeyModifiers::CONTROL));

    // Should match User priority binding (EditorCommand::Save), not plugin binding
    let cmd = registry.find_match(EditorMode::Normal);
    assert!(cmd.is_some());
    assert_eq!(cmd.unwrap(), &EditorCommand::Save, "User binding should win");
}

#[test]
fn test_plugin_binding_overrides_default_binding() {
    use crate::input::keybinding::{KeyBinding, KeySequence, KeyPattern, BindingContext, Priority};
    use std::str::FromStr;

    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register default binding for Ctrl+k
    let default_binding = KeyBinding::new(
        KeySequence::from_str("Ctrl+k").unwrap(),
        EditorCommand::DeleteChar,
        BindingContext::Global,
        Priority::Default,
    );

    registry.register(default_binding).unwrap();

    // Register plugin binding for same key with Plugin priority
    let plugin_binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+k", "plugin_cmd")
        .global()
        .build()
        .unwrap();

    registry.register_keybinding(plugin_binding).unwrap();

    assert_eq!(registry.len(), 2, "both bindings should be registered");

    // Add Ctrl+k to buffer
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('k'), KeyModifiers::CONTROL));

    // Should match Plugin priority binding (PluginCommand), not default binding
    let cmd = registry.find_match(EditorMode::Normal);
    assert!(cmd.is_some());

    match cmd.unwrap() {
        EditorCommand::PluginCommand { .. } => {
            // Plugin binding won - correct!
        }
        EditorCommand::DeleteChar => {
            panic!("Plugin binding should override Default binding");
        }
        _ => panic!("Unexpected command type"),
    }
}

#[test]
fn test_priority_ordering_plugin_between_default_and_user() {
    use crate::input::keybinding::{KeyBinding, KeySequence, KeyPattern, BindingContext, Priority};
    use std::str::FromStr;

    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    // Register all three priorities for Ctrl+s
    let default_binding = KeyBinding::new(
        KeySequence::from_str("Ctrl+s").unwrap(),
        EditorCommand::DeleteChar,
        BindingContext::Global,
        Priority::Default,
    );

    let plugin_binding = PluginBindingBuilder::new("my-plugin")
        .bind("Ctrl+s", "plugin_save")
        .global()
        .build()
        .unwrap();

    let user_binding = KeyBinding::new(
        KeySequence::from_str("Ctrl+s").unwrap(),
        EditorCommand::Save,
        BindingContext::Global,
        Priority::User,
    );

    // Register in random order
    registry.register(default_binding).unwrap();
    registry.register_keybinding(plugin_binding).unwrap();
    registry.register(user_binding).unwrap();

    assert_eq!(registry.len(), 3);

    // Add Ctrl+s to buffer
    registry.add_to_sequence(KeyPattern::new(KeyCode::Char('s'), KeyModifiers::CONTROL));

    // Should match User priority (highest)
    let cmd = registry.find_match(EditorMode::Normal);
    assert_eq!(cmd.unwrap(), &EditorCommand::Save, "User priority should win over Plugin and Default");
}
