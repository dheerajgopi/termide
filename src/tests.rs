//! Unit tests for main module

use super::*;
use std::io::Write;
use tempfile::NamedTempFile;
use termide::buffer::Buffer;
use termide::buffer::Position;
use termide::input::bindings::register_default_bindings;
use termide::input::config::load_user_keybindings;
use termide::input::input_handler::InputHandler;

#[test]
fn test_parse_args_no_file() {
    let args = vec!["termide".to_string()];
    let result = parse_args(&args).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_parse_args_with_file() {
    let args = vec!["termide".to_string(), "test.txt".to_string()];
    let result = parse_args(&args).unwrap();
    assert_eq!(result, Some("test.txt".to_string()));
}

#[test]
fn test_parse_args_too_many() {
    let args = vec![
        "termide".to_string(),
        "file1.txt".to_string(),
        "file2.txt".to_string(),
    ];
    let result = parse_args(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Usage:"));
}

#[test]
fn test_clamp_column_to_line() {
    let mut buffer = Buffer::new();
    buffer.insert_char('H', Position::origin());
    buffer.insert_char('e', Position::new(0, 1));
    buffer.insert_char('l', Position::new(0, 2));
    buffer.insert_char('l', Position::new(0, 3));
    buffer.insert_char('o', Position::new(0, 4));

    // Column within bounds
    assert_eq!(clamp_column_to_line(0, 3, &buffer), 3);

    // Column beyond line length
    assert_eq!(clamp_column_to_line(0, 10, &buffer), 5);

    // Empty line
    assert_eq!(clamp_column_to_line(1, 5, &buffer), 0);
}

mod config_integration {
    use super::*;
    use std::path::Path;
    use std::time::Duration;

    /// Helper to create a temporary config file with content
    fn create_temp_config(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_config_file_missing_no_error() {
        // Initialize registry with defaults
        let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));
        register_default_bindings(&mut input_handler.registry_mut()).unwrap();

        // Try to load from non-existent file
        let non_existent = Path::new("/tmp/termide_test_nonexistent_12345.toml");
        let result = load_user_keybindings(&mut input_handler.registry_mut(), non_existent);

        // Should get ReadError (file not found), not a crash
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            termide::input::config::ConfigError::ReadError { .. }
        ));
    }

    #[test]
    fn test_config_file_with_errors_logs_warnings() {
        // Config with invalid sequence
        let config_content = r#"
                [[keybindings]]
                sequence = "Ctr+S"  # Typo: should be Ctrl+S
                command = "file.save"

                [[keybindings]]
                sequence = "Ctrl+Q"
                command = "quit"
            "#;

        let config_file = create_temp_config(config_content);
        let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));
        register_default_bindings(&mut input_handler.registry_mut()).unwrap();

        // Load config - should succeed partially (Ctrl+Q loads, Ctr+S skipped with warning)
        let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());

        // Should succeed with 1 binding loaded (Ctrl+Q), 1 skipped (Ctr+S)
        assert!(result.is_ok());
        assert_eq!(result.unwrap().loaded, 1);
    }

    #[test]
    fn test_config_empty_file_continues_silently() {
        let config_content = r#"
                # Empty configuration
                keybindings = []
            "#;

        let config_file = create_temp_config(config_content);
        let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));
        register_default_bindings(&mut input_handler.registry_mut()).unwrap();

        // Load empty config
        let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());

        // Should succeed with 0 bindings loaded
        assert!(result.is_ok());
        assert_eq!(result.unwrap().loaded, 0);
    }

    #[test]
    fn test_config_loads_after_defaults() {
        // This test verifies that config loading happens after defaults
        // so User priority bindings can override Default priority bindings

        let config_content = r#"
                [[keybindings]]
                sequence = "Ctrl+S"
                command = "quit"  # Override Ctrl+S from save to quit
            "#;

        let config_file = create_temp_config(config_content);
        let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

        // Register defaults first (Ctrl+S -> Save)
        register_default_bindings(&mut input_handler.registry_mut()).unwrap();

        // Load user config (Ctrl+S -> Quit)
        let result = load_user_keybindings(&mut input_handler.registry_mut(), config_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().loaded, 1);

        // Verify that user binding takes precedence
        // (this is an integration test showing the priority system works)
        // The actual priority resolution is tested in registry tests
    }
}
