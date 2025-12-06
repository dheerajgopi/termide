//! Unit tests for EditorCommand string parsing

use crate::editor::EditorMode;
use crate::input::{CommandParseError, Direction, EditorCommand};
use std::str::FromStr;

// ============================================================================
// EditorCommand String Parsing Tests - Valid Commands
// ============================================================================

#[test]
fn test_parse_save_command() {
    let cmd = EditorCommand::from_str("file.save").unwrap();
    assert_eq!(cmd, EditorCommand::Save);

    // Alternative form
    let cmd = EditorCommand::from_str("save").unwrap();
    assert_eq!(cmd, EditorCommand::Save);
}

#[test]
fn test_parse_quit_command() {
    let cmd = EditorCommand::from_str("quit").unwrap();
    assert_eq!(cmd, EditorCommand::Quit);

    // Alternative form
    let cmd = EditorCommand::from_str("exit").unwrap();
    assert_eq!(cmd, EditorCommand::Quit);
}

#[test]
fn test_parse_delete_char_command() {
    let cmd = EditorCommand::from_str("delete_char").unwrap();
    assert_eq!(cmd, EditorCommand::DeleteChar);

    // Alternative forms
    let cmd = EditorCommand::from_str("delete").unwrap();
    assert_eq!(cmd, EditorCommand::DeleteChar);

    let cmd = EditorCommand::from_str("backspace").unwrap();
    assert_eq!(cmd, EditorCommand::DeleteChar);
}

#[test]
fn test_parse_move_up_command() {
    let cmd = EditorCommand::from_str("move.up").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Up));

    // Alternative forms
    let cmd = EditorCommand::from_str("move_up").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Up));

    let cmd = EditorCommand::from_str("up").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Up));
}

#[test]
fn test_parse_move_down_command() {
    let cmd = EditorCommand::from_str("move.down").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Down));

    // Alternative forms
    let cmd = EditorCommand::from_str("move_down").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Down));

    let cmd = EditorCommand::from_str("down").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Down));
}

#[test]
fn test_parse_move_left_command() {
    let cmd = EditorCommand::from_str("move.left").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Left));

    // Alternative forms
    let cmd = EditorCommand::from_str("move_left").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Left));

    let cmd = EditorCommand::from_str("left").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Left));
}

#[test]
fn test_parse_move_right_command() {
    let cmd = EditorCommand::from_str("move.right").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Right));

    // Alternative forms
    let cmd = EditorCommand::from_str("move_right").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Right));

    let cmd = EditorCommand::from_str("right").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Right));
}

#[test]
fn test_parse_mode_insert_command() {
    let cmd = EditorCommand::from_str("mode.insert").unwrap();
    assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Insert));

    // Alternative forms
    let cmd = EditorCommand::from_str("insert_mode").unwrap();
    assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Insert));

    let cmd = EditorCommand::from_str("insert").unwrap();
    assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Insert));
}

#[test]
fn test_parse_mode_normal_command() {
    let cmd = EditorCommand::from_str("mode.normal").unwrap();
    assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Normal));

    // Alternative forms
    let cmd = EditorCommand::from_str("normal_mode").unwrap();
    assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Normal));

    let cmd = EditorCommand::from_str("normal").unwrap();
    assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Normal));
}

#[test]
fn test_parse_mode_prompt_command() {
    let cmd = EditorCommand::from_str("mode.prompt").unwrap();
    assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Prompt));

    // Alternative form
    let cmd = EditorCommand::from_str("prompt_mode").unwrap();
    assert_eq!(cmd, EditorCommand::ChangeMode(EditorMode::Prompt));
}

#[test]
fn test_parse_prompt_accept_command() {
    let cmd = EditorCommand::from_str("prompt.accept").unwrap();
    assert_eq!(cmd, EditorCommand::AcceptPrompt);

    // Alternative forms
    let cmd = EditorCommand::from_str("accept_prompt").unwrap();
    assert_eq!(cmd, EditorCommand::AcceptPrompt);

    let cmd = EditorCommand::from_str("accept").unwrap();
    assert_eq!(cmd, EditorCommand::AcceptPrompt);
}

#[test]
fn test_parse_prompt_cancel_command() {
    let cmd = EditorCommand::from_str("prompt.cancel").unwrap();
    assert_eq!(cmd, EditorCommand::CancelPrompt);

    // Alternative forms
    let cmd = EditorCommand::from_str("cancel_prompt").unwrap();
    assert_eq!(cmd, EditorCommand::CancelPrompt);

    let cmd = EditorCommand::from_str("cancel").unwrap();
    assert_eq!(cmd, EditorCommand::CancelPrompt);
}

#[test]
fn test_parse_prompt_delete_char_command() {
    let cmd = EditorCommand::from_str("prompt.delete_char").unwrap();
    assert_eq!(cmd, EditorCommand::PromptDeleteChar);

    // Alternative form
    let cmd = EditorCommand::from_str("prompt_delete").unwrap();
    assert_eq!(cmd, EditorCommand::PromptDeleteChar);
}

#[test]
fn test_parse_case_insensitive() {
    // File operations
    let cmd1 = EditorCommand::from_str("FILE.SAVE").unwrap();
    let cmd2 = EditorCommand::from_str("file.save").unwrap();
    let cmd3 = EditorCommand::from_str("File.Save").unwrap();
    assert_eq!(cmd1, cmd2);
    assert_eq!(cmd2, cmd3);

    // Navigation
    let cmd1 = EditorCommand::from_str("MOVE.UP").unwrap();
    let cmd2 = EditorCommand::from_str("move.up").unwrap();
    assert_eq!(cmd1, cmd2);

    // Mode switching
    let cmd1 = EditorCommand::from_str("MODE.INSERT").unwrap();
    let cmd2 = EditorCommand::from_str("mode.insert").unwrap();
    assert_eq!(cmd1, cmd2);
}

#[test]
fn test_parse_with_whitespace() {
    // Leading whitespace
    let cmd = EditorCommand::from_str("  file.save").unwrap();
    assert_eq!(cmd, EditorCommand::Save);

    // Trailing whitespace
    let cmd = EditorCommand::from_str("quit  ").unwrap();
    assert_eq!(cmd, EditorCommand::Quit);

    // Both
    let cmd = EditorCommand::from_str("  move.up  ").unwrap();
    assert_eq!(cmd, EditorCommand::MoveCursor(Direction::Up));
}

#[test]
fn test_parse_all_navigation_commands() {
    let commands = vec![
        ("move.up", Direction::Up),
        ("move.down", Direction::Down),
        ("move.left", Direction::Left),
        ("move.right", Direction::Right),
    ];

    for (cmd_str, expected_dir) in commands {
        let cmd = EditorCommand::from_str(cmd_str).unwrap();
        assert_eq!(cmd, EditorCommand::MoveCursor(expected_dir));
    }
}

#[test]
fn test_parse_all_mode_commands() {
    let commands = vec![
        ("mode.insert", EditorMode::Insert),
        ("mode.normal", EditorMode::Normal),
        ("mode.prompt", EditorMode::Prompt),
    ];

    for (cmd_str, expected_mode) in commands {
        let cmd = EditorCommand::from_str(cmd_str).unwrap();
        assert_eq!(cmd, EditorCommand::ChangeMode(expected_mode));
    }
}

// ============================================================================
// EditorCommand String Parsing Tests - Invalid Commands
// ============================================================================

#[test]
fn test_parse_empty_string() {
    let result = EditorCommand::from_str("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CommandParseError::EmptyCommand);
}

#[test]
fn test_parse_whitespace_only() {
    let result = EditorCommand::from_str("   ");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CommandParseError::EmptyCommand);
}

#[test]
fn test_parse_unknown_command() {
    let result = EditorCommand::from_str("unknown_command");
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandParseError::UnknownCommand(cmd) => {
            assert_eq!(cmd, "unknown_command");
        }
        _ => panic!("Expected UnknownCommand error"),
    }
}

#[test]
fn test_parse_typo_in_command() {
    // Common typos - some with dots (now parsed as plugin commands), some without
    let typos_no_dot = vec!["sav", "qit", "delet"];
    let typos_with_dot = vec!["mov.up", "mode.insrt"];

    // Typos without dots should fail as UnknownCommand
    for typo in typos_no_dot {
        let result = EditorCommand::from_str(typo);
        assert!(result.is_err(), "Expected error for typo: {}", typo);
        match result.unwrap_err() {
            CommandParseError::UnknownCommand(_) => {}
            _ => panic!("Expected UnknownCommand error for typo: {}", typo),
        }
    }

    // Typos with dots are now parsed as plugin commands (unknown plugins)
    for typo in typos_with_dot {
        let result = EditorCommand::from_str(typo);
        // These should now successfully parse as plugin commands
        assert!(result.is_ok(), "Expected '{}' to parse as plugin command", typo);
        match result.unwrap() {
            EditorCommand::PluginCommand { .. } => {}
            _ => panic!("Expected PluginCommand for typo with dot: {}", typo),
        }
    }
}

#[test]
fn test_parse_incomplete_command() {
    let incomplete = vec!["file.", "move.", "mode.", "prompt."];

    for cmd in incomplete {
        let result = EditorCommand::from_str(cmd);
        assert!(result.is_err(), "Expected error for incomplete: {}", cmd);
        match result.unwrap_err() {
            // These now look like plugin commands with empty command names
            CommandParseError::InvalidPluginCommandFormat(_) => {}
            _ => panic!("Expected InvalidPluginCommandFormat error for incomplete: {}", cmd),
        }
    }
}

#[test]
fn test_parse_invalid_namespace() {
    // These are now valid plugin commands - they're just unknown plugins
    // This test now verifies they parse as plugin commands, not built-in commands
    let plugin_commands = vec![
        ("invalid.save", "invalid", "save"),
        ("badns.quit", "badns", "quit"),
        ("wrongns.up", "wrongns", "up"),
    ];

    for (cmd_str, expected_plugin, expected_cmd) in plugin_commands {
        let result = EditorCommand::from_str(cmd_str);
        assert!(result.is_ok(), "Command '{}' should parse as plugin command", cmd_str);
        match result.unwrap() {
            EditorCommand::PluginCommand { plugin_name, command_name } => {
                assert_eq!(plugin_name, expected_plugin);
                assert_eq!(command_name, expected_cmd);
            }
            _ => panic!("Expected PluginCommand for '{}'", cmd_str),
        }
    }
}

// ============================================================================
// CommandParseError Display Tests
// ============================================================================

#[test]
fn test_error_messages() {
    // Empty command
    let err = EditorCommand::from_str("").unwrap_err();
    assert_eq!(err.to_string(), "empty command string");

    // Unknown command
    let err = EditorCommand::from_str("bad_command").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown command"));
    assert!(msg.contains("bad_command"));
    assert!(msg.contains("check available commands"));
}

// ============================================================================
// Alternative Command Name Tests
// ============================================================================

#[test]
fn test_alternative_command_names_consistency() {
    // Save command alternatives
    let save1 = EditorCommand::from_str("file.save").unwrap();
    let save2 = EditorCommand::from_str("save").unwrap();
    assert_eq!(save1, save2);

    // Quit command alternatives
    let quit1 = EditorCommand::from_str("quit").unwrap();
    let quit2 = EditorCommand::from_str("exit").unwrap();
    assert_eq!(quit1, quit2);

    // Delete command alternatives
    let del1 = EditorCommand::from_str("delete_char").unwrap();
    let del2 = EditorCommand::from_str("delete").unwrap();
    let del3 = EditorCommand::from_str("backspace").unwrap();
    assert_eq!(del1, del2);
    assert_eq!(del2, del3);
}

#[test]
fn test_dot_notation_vs_underscore() {
    // Both should work for move commands
    let up1 = EditorCommand::from_str("move.up").unwrap();
    let up2 = EditorCommand::from_str("move_up").unwrap();
    assert_eq!(up1, up2);

    // Both should work for mode commands
    let insert1 = EditorCommand::from_str("mode.insert").unwrap();
    let insert2 = EditorCommand::from_str("insert_mode").unwrap();
    assert_eq!(insert1, insert2);

    // Both should work for prompt commands
    let accept1 = EditorCommand::from_str("prompt.accept").unwrap();
    let accept2 = EditorCommand::from_str("accept_prompt").unwrap();
    assert_eq!(accept1, accept2);
}

#[test]
fn test_short_form_commands() {
    // Short forms for common commands
    assert_eq!(
        EditorCommand::from_str("save").unwrap(),
        EditorCommand::Save
    );
    assert_eq!(
        EditorCommand::from_str("quit").unwrap(),
        EditorCommand::Quit
    );
    assert_eq!(
        EditorCommand::from_str("up").unwrap(),
        EditorCommand::MoveCursor(Direction::Up)
    );
    assert_eq!(
        EditorCommand::from_str("down").unwrap(),
        EditorCommand::MoveCursor(Direction::Down)
    );
    assert_eq!(
        EditorCommand::from_str("left").unwrap(),
        EditorCommand::MoveCursor(Direction::Left)
    );
    assert_eq!(
        EditorCommand::from_str("right").unwrap(),
        EditorCommand::MoveCursor(Direction::Right)
    );
    assert_eq!(
        EditorCommand::from_str("insert").unwrap(),
        EditorCommand::ChangeMode(EditorMode::Insert)
    );
    assert_eq!(
        EditorCommand::from_str("normal").unwrap(),
        EditorCommand::ChangeMode(EditorMode::Normal)
    );
}

#[test]
fn test_parse_delete_forward_command() {
    let cmd = EditorCommand::from_str("delete_forward").unwrap();
    assert_eq!(cmd, EditorCommand::DeleteForward);

    // Alternative forms
    let cmd = EditorCommand::from_str("delete.forward").unwrap();
    assert_eq!(cmd, EditorCommand::DeleteForward);

    let cmd = EditorCommand::from_str("del").unwrap();
    assert_eq!(cmd, EditorCommand::DeleteForward);
}

#[test]
fn test_parse_move_to_line_start_command() {
    let cmd = EditorCommand::from_str("move.line_start").unwrap();
    assert_eq!(cmd, EditorCommand::MoveToLineStart);

    // Alternative forms
    let cmd = EditorCommand::from_str("move_line_start").unwrap();
    assert_eq!(cmd, EditorCommand::MoveToLineStart);

    let cmd = EditorCommand::from_str("line_start").unwrap();
    assert_eq!(cmd, EditorCommand::MoveToLineStart);

    let cmd = EditorCommand::from_str("home").unwrap();
    assert_eq!(cmd, EditorCommand::MoveToLineStart);
}

#[test]
fn test_parse_move_to_line_end_command() {
    let cmd = EditorCommand::from_str("move.line_end").unwrap();
    assert_eq!(cmd, EditorCommand::MoveToLineEnd);

    // Alternative forms
    let cmd = EditorCommand::from_str("move_line_end").unwrap();
    assert_eq!(cmd, EditorCommand::MoveToLineEnd);

    let cmd = EditorCommand::from_str("line_end").unwrap();
    assert_eq!(cmd, EditorCommand::MoveToLineEnd);

    let cmd = EditorCommand::from_str("end").unwrap();
    assert_eq!(cmd, EditorCommand::MoveToLineEnd);
}

#[test]
fn test_parse_page_up_command() {
    let cmd = EditorCommand::from_str("page.up").unwrap();
    assert_eq!(cmd, EditorCommand::PageUp);

    // Alternative forms
    let cmd = EditorCommand::from_str("page_up").unwrap();
    assert_eq!(cmd, EditorCommand::PageUp);

    let cmd = EditorCommand::from_str("pageup").unwrap();
    assert_eq!(cmd, EditorCommand::PageUp);
}

#[test]
fn test_parse_page_down_command() {
    let cmd = EditorCommand::from_str("page.down").unwrap();
    assert_eq!(cmd, EditorCommand::PageDown);

    // Alternative forms
    let cmd = EditorCommand::from_str("page_down").unwrap();
    assert_eq!(cmd, EditorCommand::PageDown);

    let cmd = EditorCommand::from_str("pagedown").unwrap();
    assert_eq!(cmd, EditorCommand::PageDown);
}

#[test]
fn test_parse_insert_tab_command() {
    let cmd = EditorCommand::from_str("insert_tab").unwrap();
    assert_eq!(cmd, EditorCommand::InsertTab);

    // Alternative form
    let cmd = EditorCommand::from_str("tab").unwrap();
    assert_eq!(cmd, EditorCommand::InsertTab);
}

// ============================================================================
// Placeholder Commands - Selection Operations (Future Features)
// ============================================================================

#[test]
fn test_parse_select_left_command() {
    let cmd = EditorCommand::from_str("select.left").unwrap();
    assert_eq!(cmd, EditorCommand::SelectLeft);

    // Alternative form
    let cmd = EditorCommand::from_str("select_left").unwrap();
    assert_eq!(cmd, EditorCommand::SelectLeft);
}

#[test]
fn test_parse_select_right_command() {
    let cmd = EditorCommand::from_str("select.right").unwrap();
    assert_eq!(cmd, EditorCommand::SelectRight);

    // Alternative form
    let cmd = EditorCommand::from_str("select_right").unwrap();
    assert_eq!(cmd, EditorCommand::SelectRight);
}

#[test]
fn test_parse_select_up_command() {
    let cmd = EditorCommand::from_str("select.up").unwrap();
    assert_eq!(cmd, EditorCommand::SelectUp);

    // Alternative form
    let cmd = EditorCommand::from_str("select_up").unwrap();
    assert_eq!(cmd, EditorCommand::SelectUp);
}

#[test]
fn test_parse_select_down_command() {
    let cmd = EditorCommand::from_str("select.down").unwrap();
    assert_eq!(cmd, EditorCommand::SelectDown);

    // Alternative form
    let cmd = EditorCommand::from_str("select_down").unwrap();
    assert_eq!(cmd, EditorCommand::SelectDown);
}

#[test]
fn test_parse_select_line_start_command() {
    let cmd = EditorCommand::from_str("select.line_start").unwrap();
    assert_eq!(cmd, EditorCommand::SelectLineStart);

    // Alternative form
    let cmd = EditorCommand::from_str("select_line_start").unwrap();
    assert_eq!(cmd, EditorCommand::SelectLineStart);
}

#[test]
fn test_parse_select_line_end_command() {
    let cmd = EditorCommand::from_str("select.line_end").unwrap();
    assert_eq!(cmd, EditorCommand::SelectLineEnd);

    // Alternative form
    let cmd = EditorCommand::from_str("select_line_end").unwrap();
    assert_eq!(cmd, EditorCommand::SelectLineEnd);
}

#[test]
fn test_parse_select_all_command() {
    let cmd = EditorCommand::from_str("select.all").unwrap();
    assert_eq!(cmd, EditorCommand::SelectAll);

    // Alternative form
    let cmd = EditorCommand::from_str("select_all").unwrap();
    assert_eq!(cmd, EditorCommand::SelectAll);
}

// ============================================================================
// Placeholder Commands - Clipboard Operations (Future Features)
// ============================================================================

#[test]
fn test_parse_copy_command() {
    let cmd = EditorCommand::from_str("copy").unwrap();
    assert_eq!(cmd, EditorCommand::Copy);
}

#[test]
fn test_parse_cut_command() {
    let cmd = EditorCommand::from_str("cut").unwrap();
    assert_eq!(cmd, EditorCommand::Cut);
}

#[test]
fn test_parse_paste_command() {
    let cmd = EditorCommand::from_str("paste").unwrap();
    assert_eq!(cmd, EditorCommand::Paste);
}

// ============================================================================
// Plugin Command Parsing Tests
// ============================================================================

#[test]
fn test_parse_valid_plugin_command() {
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
}

#[test]
fn test_parse_plugin_command_with_hyphen() {
    let cmd = EditorCommand::from_str("my-plugin.my_command").unwrap();
    match cmd {
        EditorCommand::PluginCommand {
            plugin_name,
            command_name,
        } => {
            assert_eq!(plugin_name, "my-plugin");
            assert_eq!(command_name, "my_command");
        }
        _ => panic!("Expected PluginCommand variant"),
    }
}

#[test]
fn test_parse_plugin_command_with_numbers() {
    let cmd = EditorCommand::from_str("plugin123.cmd456").unwrap();
    match cmd {
        EditorCommand::PluginCommand {
            plugin_name,
            command_name,
        } => {
            assert_eq!(plugin_name, "plugin123");
            assert_eq!(command_name, "cmd456");
        }
        _ => panic!("Expected PluginCommand variant"),
    }
}

#[test]
fn test_parse_plugin_command_case_insensitive() {
    let cmd = EditorCommand::from_str("MyPlugin.MyCommand").unwrap();
    match cmd {
        EditorCommand::PluginCommand {
            plugin_name,
            command_name,
        } => {
            // Case is preserved in plugin/command names after lowercasing
            assert_eq!(plugin_name, "myplugin");
            assert_eq!(command_name, "mycommand");
        }
        _ => panic!("Expected PluginCommand variant"),
    }
}

#[test]
fn test_parse_plugin_command_with_whitespace() {
    let cmd = EditorCommand::from_str("  my_plugin.my_cmd  ").unwrap();
    match cmd {
        EditorCommand::PluginCommand {
            plugin_name,
            command_name,
        } => {
            assert_eq!(plugin_name, "my_plugin");
            assert_eq!(command_name, "my_cmd");
        }
        _ => panic!("Expected PluginCommand variant"),
    }
}

#[test]
fn test_parse_plugin_command_no_dot() {
    let result = EditorCommand::from_str("plugincommand");
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandParseError::UnknownCommand(cmd) => {
            assert_eq!(cmd, "plugincommand");
        }
        _ => panic!("Expected UnknownCommand error"),
    }
}

#[test]
fn test_parse_plugin_command_empty_plugin_name() {
    let result = EditorCommand::from_str(".command");
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandParseError::InvalidPluginCommandFormat(cmd) => {
            assert_eq!(cmd, ".command");
        }
        _ => panic!("Expected InvalidPluginCommandFormat error"),
    }
}

#[test]
fn test_parse_plugin_command_empty_command_name() {
    let result = EditorCommand::from_str("plugin.");
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandParseError::InvalidPluginCommandFormat(cmd) => {
            assert_eq!(cmd, "plugin.");
        }
        _ => panic!("Expected InvalidPluginCommandFormat error"),
    }
}

#[test]
fn test_parse_plugin_command_too_many_dots() {
    let result = EditorCommand::from_str("too.many.dots");
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandParseError::InvalidPluginCommandFormat(cmd) => {
            assert_eq!(cmd, "too.many.dots");
        }
        _ => panic!("Expected InvalidPluginCommandFormat error"),
    }
}

#[test]
fn test_parse_plugin_command_empty_segment() {
    let result = EditorCommand::from_str("plugin..command");
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandParseError::InvalidPluginCommandFormat(cmd) => {
            assert_eq!(cmd, "plugin..command");
        }
        _ => panic!("Expected InvalidPluginCommandFormat error"),
    }
}

#[test]
fn test_parse_plugin_command_invalid_chars_in_plugin_name() {
    // Special characters not allowed in plugin names (except hyphen and underscore)
    let invalid = vec!["plugin@name.cmd", "plugin name.cmd", "plugin$name.cmd"];

    for cmd_str in invalid {
        let result = EditorCommand::from_str(cmd_str);
        assert!(
            result.is_err(),
            "Expected error for invalid plugin name: {}",
            cmd_str
        );
        match result.unwrap_err() {
            CommandParseError::InvalidPluginCommandFormat(_) => {}
            _ => panic!(
                "Expected InvalidPluginCommandFormat error for: {}",
                cmd_str
            ),
        }
    }
}

#[test]
fn test_parse_plugin_command_invalid_chars_in_command_name() {
    // Hyphens not allowed in command names (only underscore)
    let invalid = vec!["plugin.cmd-name", "plugin.cmd name", "plugin.cmd$name"];

    for cmd_str in invalid {
        let result = EditorCommand::from_str(cmd_str);
        assert!(
            result.is_err(),
            "Expected error for invalid command name: {}",
            cmd_str
        );
        match result.unwrap_err() {
            CommandParseError::InvalidPluginCommandFormat(_) => {}
            _ => panic!(
                "Expected InvalidPluginCommandFormat error for: {}",
                cmd_str
            ),
        }
    }
}

#[test]
fn test_plugin_command_error_message() {
    let err = EditorCommand::from_str("bad..format").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("invalid plugin command format"));
    assert!(msg.contains("bad..format"));
    assert!(msg.contains("plugin_name.command_name"));
}

#[test]
fn test_existing_dotted_commands_not_parsed_as_plugin() {
    // These should still parse as their original commands, not plugin commands
    let existing = vec![
        ("file.save", EditorCommand::Save),
        ("move.up", EditorCommand::MoveCursor(Direction::Up)),
        ("mode.insert", EditorCommand::ChangeMode(EditorMode::Insert)),
        ("prompt.accept", EditorCommand::AcceptPrompt),
    ];

    for (cmd_str, expected) in existing {
        let cmd = EditorCommand::from_str(cmd_str).unwrap();
        assert_eq!(
            cmd, expected,
            "Command '{}' should parse as built-in, not plugin command",
            cmd_str
        );
    }
}

#[test]
fn test_plugin_command_examples() {
    // Test realistic plugin command examples
    let examples = vec![
        ("rust_analyzer.format", "rust_analyzer", "format"),
        ("lsp.goto_definition", "lsp", "goto_definition"),
        ("git-blame.show", "git-blame", "show"),
        ("formatter.run", "formatter", "run"),
    ];

    for (cmd_str, expected_plugin, expected_cmd) in examples {
        let cmd = EditorCommand::from_str(cmd_str).unwrap();
        match cmd {
            EditorCommand::PluginCommand {
                plugin_name,
                command_name,
            } => {
                assert_eq!(plugin_name, expected_plugin);
                assert_eq!(command_name, expected_cmd);
            }
            _ => panic!(
                "Expected PluginCommand for '{}', got {:?}",
                cmd_str, cmd
            ),
        }
    }
}
