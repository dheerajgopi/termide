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
    // Common typos
    let typos = vec!["sav", "qit", "delet", "mov.up", "mode.insrt"];

    for typo in typos {
        let result = EditorCommand::from_str(typo);
        assert!(result.is_err(), "Expected error for typo: {}", typo);
        match result.unwrap_err() {
            CommandParseError::UnknownCommand(_) => {}
            _ => panic!("Expected UnknownCommand error for typo: {}", typo),
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
            CommandParseError::UnknownCommand(_) => {}
            _ => panic!("Expected UnknownCommand error for incomplete: {}", cmd),
        }
    }
}

#[test]
fn test_parse_invalid_namespace() {
    let invalid = vec!["invalid.save", "badns.quit", "wrongns.up"];

    for cmd in invalid {
        let result = EditorCommand::from_str(cmd);
        assert!(result.is_err(), "Expected error for invalid: {}", cmd);
        match result.unwrap_err() {
            CommandParseError::UnknownCommand(_) => {}
            _ => panic!("Expected UnknownCommand error for invalid: {}", cmd),
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
