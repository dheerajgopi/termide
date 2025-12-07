//! Unit tests for config module

use crate::editor::EditorMode;
use crate::input::config::{load_user_keybindings, parse_mode, ConfigError};
use crate::input::registry::KeyBindingRegistry;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tempfile::NamedTempFile;

/// Helper to create a temporary config file with content
fn create_temp_config(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

#[test]
fn test_parse_mode_valid() {
    assert_eq!(parse_mode("insert").unwrap(), EditorMode::Insert);
    assert_eq!(parse_mode("normal").unwrap(), EditorMode::Normal);
    assert_eq!(parse_mode("prompt").unwrap(), EditorMode::Prompt);
}

#[test]
fn test_parse_mode_case_insensitive() {
    assert_eq!(parse_mode("INSERT").unwrap(), EditorMode::Insert);
    assert_eq!(parse_mode("Normal").unwrap(), EditorMode::Normal);
    assert_eq!(parse_mode("PROMPT").unwrap(), EditorMode::Prompt);
}

#[test]
fn test_parse_mode_whitespace() {
    assert_eq!(parse_mode("  insert  ").unwrap(), EditorMode::Insert);
    assert_eq!(parse_mode("\tnormal\t").unwrap(), EditorMode::Normal);
}

#[test]
fn test_parse_mode_invalid() {
    assert!(parse_mode("invalid").is_err());
    assert!(parse_mode("insrt").is_err()); // typo
    assert!(parse_mode("").is_err());
}

#[test]
fn test_load_empty_config() {
    let config_content = r#"
        # Empty configuration
        keybindings = []
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 0);
}

#[test]
fn test_load_valid_single_binding() {
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+S"
        command = "file.save"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_load_mode_specific_binding() {
    let config_content = r#"
        [[keybindings]]
        sequence = "i"
        command = "mode.insert"
        mode = "normal"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);
}

#[test]
fn test_load_multiple_bindings() {
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+S"
        command = "file.save"

        [[keybindings]]
        sequence = "Ctrl+Q"
        command = "quit"

        [[keybindings]]
        sequence = "i"
        command = "mode.insert"
        mode = "normal"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 3);
    assert_eq!(registry.len(), 3);
}

#[test]
fn test_load_multikey_sequence() {
    let config_content = r#"
        [[keybindings]]
        sequence = "d d"
        command = "delete_char"
        mode = "normal"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);
}

#[test]
fn test_load_invalid_toml() {
    let config_content = r#"
        this is not valid TOML
        [[keybindings
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ConfigError::TomlParseError { .. }
    ));
}

#[test]
fn test_load_invalid_sequence_continues() {
    // Invalid sequence should log warning but continue with other bindings
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+"
        command = "file.save"

        [[keybindings]]
        sequence = "Ctrl+Q"
        command = "quit"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    // Only the valid binding should be loaded
    assert_eq!(result.unwrap().loaded, 1);
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_load_invalid_command_continues() {
    // Invalid command should log warning but continue
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+X"
        command = "unknown_command"

        [[keybindings]]
        sequence = "Ctrl+S"
        command = "file.save"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);
}

#[test]
fn test_load_invalid_mode_continues() {
    let config_content = r#"
        [[keybindings]]
        sequence = "i"
        command = "mode.insert"
        mode = "invalid_mode"

        [[keybindings]]
        sequence = "Ctrl+S"
        command = "file.save"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);
}

#[test]
fn test_priority_is_user() {
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+S"
        command = "file.save"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    load_user_keybindings(&mut registry, temp_file.path()).unwrap();

    // Verify the binding was registered (we can't directly check priority,
    // but we can verify it was added)
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_file_not_found() {
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));
    let result = load_user_keybindings(&mut registry, Path::new("/nonexistent/path.toml"));

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::ReadError { .. }));
}

#[test]
fn test_mixed_valid_invalid_bindings() {
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+S"
        command = "file.save"

        [[keybindings]]
        sequence = "InvalidSeq+"
        command = "quit"

        [[keybindings]]
        sequence = "Ctrl+Q"
        command = "quit"

        [[keybindings]]
        sequence = "i"
        command = "unknown_cmd"
        mode = "normal"

        [[keybindings]]
        sequence = "Esc"
        command = "mode.normal"
        mode = "insert"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    // Should load 3 valid bindings (Ctrl+S, Ctrl+Q, Esc)
    assert_eq!(result.unwrap().loaded, 3);
    assert_eq!(registry.len(), 3);
}

#[test]
fn test_global_binding_no_mode() {
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+S"
        command = "file.save"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);
}

#[test]
fn test_case_insensitive_mode() {
    let config_content = r#"
        [[keybindings]]
        sequence = "i"
        command = "mode.insert"
        mode = "NORMAL"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);
}

#[test]
fn test_whitespace_in_mode() {
    let config_content = r#"
        [[keybindings]]
        sequence = "i"
        command = "mode.insert"
        mode = "  normal  "
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);
}

#[test]
fn test_plugin_command() {
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+F"
        command = "rust_analyzer.format"
        mode = "normal"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 1);
}

#[test]
fn test_empty_sequence_error() {
    let config_content = r#"
        [[keybindings]]
        sequence = ""
        command = "file.save"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 0); // Invalid entry skipped
}

#[test]
fn test_empty_command_error() {
    let config_content = r#"
        [[keybindings]]
        sequence = "Ctrl+S"
        command = ""
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 0); // Invalid entry skipped
}

#[test]
fn test_all_modes() {
    let config_content = r#"
        [[keybindings]]
        sequence = "a"
        command = "mode.insert"
        mode = "insert"

        [[keybindings]]
        sequence = "b"
        command = "mode.normal"
        mode = "normal"

        [[keybindings]]
        sequence = "c"
        command = "mode.prompt"
        mode = "prompt"
    "#;
    let temp_file = create_temp_config(config_content);
    let mut registry = KeyBindingRegistry::new(Duration::from_secs(1));

    let result = load_user_keybindings(&mut registry, temp_file.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().loaded, 3);
}
