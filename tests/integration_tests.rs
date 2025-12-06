//! Integration tests for TermIDE editor
//!
//! These tests verify complete workflows: opening files, editing, saving, and quitting.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use termide::buffer::Position;
use termide::editor::{EditorMode, EditorState};
use termide::file_io::{read_file, write_file};
use termide::input::EditorCommand;
use termide::input::input_handler::InputHandler;
use termide::input::keybinding::{KeySequence, KeyBinding, BindingContext, Priority};
use std::time::Duration;

/// Test: Open existing file → verify content loaded correctly
#[test]
fn test_open_existing_file() {
    // Arrange: Create a temp file with known content
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.txt");
    let content = "Hello, World!\nThis is a test.";
    write_file(&file_path, content).unwrap();

    // Act: Load the file into editor state
    let state = EditorState::from_file(&file_path).unwrap();

    // Assert: Content matches
    assert_eq!(state.buffer().content(), content);
    assert!(!state.buffer().is_dirty());
    assert_eq!(state.mode(), EditorMode::Insert);
    assert_eq!(
        state.buffer().file_path(),
        Some(&file_path)
    );
}

/// Test: Edit buffer → save → verify file on disk updated
#[test]
fn test_edit_and_save() {
    // Arrange: Create temp file
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("edit_test.txt");
    write_file(&file_path, "Initial content").unwrap();

    // Act: Load, edit, and save
    let mut state = EditorState::from_file(&file_path).unwrap();
    state.handle_char_insert('\n', Position::new(0, 15)); // Add newline at end
    state.handle_char_insert('N', Position::new(1, 0));
    state.handle_char_insert('e', Position::new(1, 1));
    state.handle_char_insert('w', Position::new(1, 2));

    assert!(state.buffer().is_dirty());
    state.save().unwrap();
    assert!(!state.buffer().is_dirty());

    // Assert: File on disk matches buffer
    let saved_content = read_file(&file_path).unwrap();
    assert_eq!(saved_content, "Initial content\nNew");
    assert_eq!(state.status_message(), Some("Saved successfully"));
}

/// Test: Edit buffer → quit without save → verify file unchanged
#[test]
fn test_quit_without_save() {
    // Arrange: Create temp file
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("quit_test.txt");
    let original_content = "Original content";
    write_file(&file_path, original_content).unwrap();

    // Act: Load, edit, but don't save
    let mut state = EditorState::from_file(&file_path).unwrap();
    state.handle_char_insert('X', Position::new(0, 0));

    assert!(state.buffer().is_dirty());

    // First quit attempt - should warn
    let can_quit = state.request_quit();
    assert!(!can_quit);
    assert!(!state.should_quit());
    assert!(state.status_message().unwrap().contains("Unsaved changes"));

    // Second quit attempt - should force quit
    let can_quit = state.request_quit();
    assert!(can_quit);
    assert!(state.should_quit());

    // Assert: File on disk unchanged
    let saved_content = read_file(&file_path).unwrap();
    assert_eq!(saved_content, original_content);
}

/// Test: Create new file → type content → save → verify file created
#[test]
fn test_create_new_file() {
    // Arrange: Use path that doesn't exist
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("new_file.txt");
    assert!(!file_path.exists());

    // Act: Create editor state with non-existent file path
    let mut state = EditorState::from_file(&file_path).unwrap();
    assert_eq!(state.buffer().content(), "");

    // Type some content
    state.handle_char_insert('H', Position::origin());
    state.handle_char_insert('i', Position::new(0, 1));

    // Save
    state.save().unwrap();

    // Assert: File now exists with correct content
    assert!(file_path.exists());
    let saved_content = read_file(&file_path).unwrap();
    assert_eq!(saved_content, "Hi");
}

/// Test: Handle file with various encodings (UTF-8, unicode characters)
#[test]
fn test_unicode_content() {
    // Arrange: Create file with unicode content
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("unicode.txt");
    let unicode_content = "Hello 世界 🌍\nРусский текст\n日本語";
    write_file(&file_path, unicode_content).unwrap();

    // Act: Load and verify
    let state = EditorState::from_file(&file_path).unwrap();

    // Assert: Unicode content preserved
    assert_eq!(state.buffer().content(), unicode_content);
    assert_eq!(state.buffer().line_count(), 3);
}

/// Test: Large file (1000 lines) loads and edits smoothly
#[test]
fn test_large_file() {
    // Arrange: Create large file
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("large.txt");

    let mut large_content = String::new();
    for i in 0..1000 {
        large_content.push_str(&format!("Line {}\n", i));
    }
    write_file(&file_path, &large_content).unwrap();

    // Act: Load file
    let mut state = EditorState::from_file(&file_path).unwrap();

    // Assert: All lines loaded (1000 lines + 1 empty line at end)
    assert_eq!(state.buffer().line_count(), 1001);

    // Edit and save - insert at start of line 500
    state.handle_char_insert('X', Position::new(500, 0));
    state.save().unwrap();

    // Verify edit persisted
    let reloaded = EditorState::from_file(&file_path).unwrap();
    assert_eq!(
        reloaded.buffer().get_line(500),
        Some("XLine 500\n".to_string())
    );
}

/// Test: Special characters (tabs, newlines, emojis) handled correctly
#[test]
fn test_special_characters() {
    // Arrange
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("special.txt");

    let mut state = EditorState::from_file(&file_path).unwrap();

    // Act: Insert various special characters
    state.handle_char_insert('\t', Position::origin()); // Tab
    state.handle_char_insert('H', Position::new(0, 1));
    state.handle_char_insert('\n', Position::new(0, 2)); // Newline
    state.handle_char_insert('🚀', Position::new(1, 0)); // Emoji

    state.save().unwrap();

    // Assert: Special characters preserved
    let saved_content = read_file(&file_path).unwrap();
    assert_eq!(saved_content, "\tH\n🚀");
}

/// Test: File permission errors display appropriate messages
#[test]
#[cfg(unix)] // Permission tests only work on Unix systems
fn test_permission_error() {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    // Arrange: Create read-only directory (prevents atomic write from creating temp file)
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("file.txt");
    write_file(&file_path, "Original").unwrap();

    // Make directory read-only (prevents creating new files)
    let permissions = Permissions::from_mode(0o555);
    fs::set_permissions(dir.path(), permissions).unwrap();

    // Act: Try to save changes (will fail because can't create temp file)
    let mut state = EditorState::from_file(&file_path).unwrap();
    state.handle_char_insert('X', Position::origin());

    let result = state.save();

    // Cleanup: restore permissions first so temp dir can be deleted
    let permissions = Permissions::from_mode(0o755);
    fs::set_permissions(dir.path(), permissions).unwrap();

    // Assert: Error occurred
    assert!(result.is_err());
    let error_msg = format!("{:#}", result.unwrap_err());
    assert!(
        error_msg.contains("Permission denied") || error_msg.contains("permission") || error_msg.contains("Failed to save"),
        "Expected permission error, got: {}",
        error_msg
    );
}

/// Test: Invalid file path handled gracefully
#[test]
fn test_invalid_path() {
    // Arrange: Path in non-existent directory
    let invalid_path = PathBuf::from("/non/existent/directory/file.txt");

    // Act: Try to create editor state
    let mut state = EditorState::from_file(&invalid_path).unwrap();
    state.handle_char_insert('X', Position::origin());

    // Saving to invalid path should error
    let result = state.save();

    // Assert: Error occurred with helpful message
    assert!(result.is_err());
    let error_msg = format!("{:#}", result.unwrap_err());
    assert!(
        error_msg.contains("Failed to save") || error_msg.contains("No such file"),
        "Expected file error, got: {}",
        error_msg
    );
}

/// Test: Mode switching during edit workflow
#[test]
fn test_mode_switching_workflow() {
    let mut state = EditorState::new();

    // Start in Insert mode
    assert_eq!(state.mode(), EditorMode::Insert);

    // Switch to Normal mode
    state.set_mode(EditorMode::Normal);
    assert_eq!(state.mode(), EditorMode::Normal);

    // Switch back to Insert mode
    state.set_mode(EditorMode::Insert);
    assert_eq!(state.mode(), EditorMode::Insert);
}

/// Test: Empty buffer handling
#[test]
fn test_empty_buffer() {
    let mut state = EditorState::new();

    assert_eq!(state.buffer().content(), "");
    assert_eq!(state.buffer().line_count(), 1); // Empty buffer has 1 line
    assert!(!state.buffer().is_dirty());

    // Quit with empty buffer should work immediately
    assert!(state.request_quit());
    assert!(state.should_quit());
}

/// Test: Multiline editing workflow
#[test]
fn test_multiline_editing() {
    let mut state = EditorState::new();

    // Type multi-line content
    state.handle_char_insert('L', Position::origin());
    state.handle_char_insert('1', Position::new(0, 1));
    state.handle_char_insert('\n', Position::new(0, 2));
    state.handle_char_insert('L', Position::new(1, 0));
    state.handle_char_insert('2', Position::new(1, 1));
    state.handle_char_insert('\n', Position::new(1, 2));
    state.handle_char_insert('L', Position::new(2, 0));
    state.handle_char_insert('3', Position::new(2, 1));

    assert_eq!(state.buffer().content(), "L1\nL2\nL3");
    assert_eq!(state.buffer().line_count(), 3);

    // Delete at position (0, 2) - deletes the newline, joining lines 0 and 1
    state.handle_char_delete(Position::new(0, 2));
    assert_eq!(state.buffer().content(), "L1L2\nL3");

    // Delete at position (0, 2) - deletes the 'L' at that position
    state.handle_char_delete(Position::new(0, 2));
    assert_eq!(state.buffer().content(), "L12\nL3");
}

/// Test: Status message management
#[test]
fn test_status_messages() {
    let mut state = EditorState::new();

    // No initial message
    assert_eq!(state.status_message(), None);

    // Set message
    state.set_status_message("Test message".to_string());
    assert_eq!(state.status_message(), Some("Test message"));

    // Clear message
    state.clear_status_message();
    assert_eq!(state.status_message(), None);
}

/// Test: Dirty flag tracking through workflow
#[test]
fn test_dirty_flag_workflow() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("dirty_test.txt");

    // Create new buffer - not dirty
    let mut state = EditorState::from_file(&file_path).unwrap();
    assert!(!state.buffer().is_dirty());

    // Edit - becomes dirty
    state.handle_char_insert('X', Position::origin());
    assert!(state.buffer().is_dirty());

    // Save - no longer dirty
    state.save().unwrap();
    assert!(!state.buffer().is_dirty());

    // Edit again - dirty again
    state.handle_char_insert('Y', Position::new(0, 1));
    assert!(state.buffer().is_dirty());
}

// ============================================================================
// Plugin Command Integration Tests
// ============================================================================

/// Test: Parse and register plugin command binding
#[test]
fn test_plugin_command_parsing() {
    use std::str::FromStr;

    // Parse valid plugin command
    let cmd = EditorCommand::from_str("rust_analyzer.format").unwrap();
    match cmd {
        EditorCommand::PluginCommand {
            plugin_name,
            command_name,
        } => {
            assert_eq!(plugin_name, "rust_analyzer");
            assert_eq!(command_name, "format");
        }
        _ => panic!("Expected PluginCommand variant"),
    }

    // Parse another plugin command
    let cmd = EditorCommand::from_str("lsp.goto_definition").unwrap();
    match cmd {
        EditorCommand::PluginCommand {
            plugin_name,
            command_name,
        } => {
            assert_eq!(plugin_name, "lsp");
            assert_eq!(command_name, "goto_definition");
        }
        _ => panic!("Expected PluginCommand variant"),
    }
}

/// Test: Register plugin command binding and trigger it
#[test]
fn test_register_and_trigger_plugin_command() {
    use std::str::FromStr;
    use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};

    // Create input handler
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register a plugin command binding
    let sequence = KeySequence::from_str("F5").unwrap();
    let command = EditorCommand::from_str("rust_analyzer.format").unwrap();
    let binding = KeyBinding::new(
        sequence,
        command.clone(),
        BindingContext::Global,
        Priority::Plugin,
    );

    input_handler
        .registry_mut()
        .register(binding)
        .unwrap();

    // Simulate the key event
    let key_event = KeyEvent::new(
        KeyCode::F(5),
        KeyModifiers::NONE,
    );

    // Process the key event
    let result = input_handler.process_key_event(key_event, EditorMode::Normal);

    // Verify the command was matched
    use termide::input::input_handler::MatchResult;
    match result {
        MatchResult::Matched(matched_cmd) => {
            match matched_cmd {
                EditorCommand::PluginCommand {
                    plugin_name,
                    command_name,
                } => {
                    assert_eq!(plugin_name, "rust_analyzer");
                    assert_eq!(command_name, "format");
                }
                _ => panic!("Expected PluginCommand, got {:?}", matched_cmd),
            }
        }
        _ => panic!("Expected Matched result, got {:?}", result),
    }
}

/// Test: Plugin command with invalid format rejected
#[test]
fn test_plugin_command_invalid_format() {
    use std::str::FromStr;

    // Too many dots
    let result = EditorCommand::from_str("too.many.dots");
    assert!(result.is_err());

    // Empty plugin name
    let result = EditorCommand::from_str(".command");
    assert!(result.is_err());

    // Empty command name
    let result = EditorCommand::from_str("plugin.");
    assert!(result.is_err());

    // Invalid characters in plugin name
    let result = EditorCommand::from_str("plugin@name.cmd");
    assert!(result.is_err());

    // Invalid characters in command name (hyphen not allowed)
    let result = EditorCommand::from_str("plugin.cmd-name");
    assert!(result.is_err());
}

/// Test: Multiple plugin commands from different plugins
#[test]
fn test_multiple_plugin_commands() {
    use std::str::FromStr;
    use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};

    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register multiple plugin commands (use F-keys to avoid conflicts)
    let bindings = vec![
        ("F5", "rust_analyzer.format", "rust_analyzer", "format", 5),
        ("F6", "lsp.goto_definition", "lsp", "goto_definition", 6),
        ("F7", "my-plugin.refactor", "my-plugin", "refactor", 7),
    ];

    for (key_str, cmd_str, _expected_plugin, _expected_cmd, f_num) in &bindings {
        let sequence = KeySequence::from_str(key_str).unwrap();
        let command = EditorCommand::from_str(cmd_str).unwrap();
        let binding = KeyBinding::new(
            sequence,
            command,
            BindingContext::Global,
            Priority::Plugin,
        );
        input_handler
            .registry_mut()
            .register(binding)
            .unwrap();
    }

    // Test first binding
    let key_event = KeyEvent::new(
        KeyCode::F(5),
        KeyModifiers::NONE,
    );
    let result = input_handler.process_key_event(key_event, EditorMode::Normal);
    use termide::input::input_handler::MatchResult;
    match result {
        MatchResult::Matched(EditorCommand::PluginCommand {
            plugin_name,
            command_name,
        }) => {
            assert_eq!(plugin_name, "rust_analyzer");
            assert_eq!(command_name, "format");
        }
        _ => panic!("Expected plugin command match"),
    }

    // Test second binding
    let key_event = KeyEvent::new(
        KeyCode::F(6),
        KeyModifiers::NONE,
    );
    let result = input_handler.process_key_event(key_event, EditorMode::Normal);
    match result {
        MatchResult::Matched(EditorCommand::PluginCommand {
            plugin_name,
            command_name,
        }) => {
            assert_eq!(plugin_name, "lsp");
            assert_eq!(command_name, "goto_definition");
        }
        _ => panic!("Expected plugin command match"),
    }

    // Test third binding
    let key_event = KeyEvent::new(
        KeyCode::F(7),
        KeyModifiers::NONE,
    );
    let result = input_handler.process_key_event(key_event, EditorMode::Normal);
    match result {
        MatchResult::Matched(EditorCommand::PluginCommand {
            plugin_name,
            command_name,
        }) => {
            assert_eq!(plugin_name, "my-plugin");
            assert_eq!(command_name, "refactor");
        }
        _ => panic!("Expected plugin command match"),
    }
}

/// Test: Plugin commands work with mode-specific contexts
#[test]
fn test_plugin_command_mode_specific() {
    use std::str::FromStr;
    use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};

    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));

    // Register plugin command only active in Normal mode
    let sequence = KeySequence::from_str("F8").unwrap();
    let command = EditorCommand::from_str("formatter.run").unwrap();
    let binding = KeyBinding::new(
        sequence,
        command,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Plugin,
    );

    input_handler
        .registry_mut()
        .register(binding)
        .unwrap();

    let key_event = KeyEvent::new(KeyCode::F(8), KeyModifiers::NONE);

    // Should match in Normal mode
    let result = input_handler.process_key_event(key_event, EditorMode::Normal);
    use termide::input::input_handler::MatchResult;
    match result {
        MatchResult::Matched(EditorCommand::PluginCommand { .. }) => {}
        _ => panic!("Expected match in Normal mode"),
    }

    // Should NOT match in Insert mode
    let result = input_handler.process_key_event(key_event, EditorMode::Insert);
    match result {
        MatchResult::NoMatch => {}
        _ => panic!("Expected NoMatch in Insert mode"),
    }
}
