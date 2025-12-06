//! TermIDE - A terminal-based text editor
//!
//! A fast, efficient text editor built with Rust, designed for terminal environments.

use std::env;
use std::io;
use std::panic;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use termide::buffer::Position;
use termide::editor::{EditorMode, EditorState};
use termide::input::{Direction, EditorCommand};
use termide::input::bindings::register_default_bindings;
use termide::input::config::{get_config_path, load_user_keybindings};
use termide::input::input_handler::{InputHandler, MatchResult};
use termide::ui::Renderer;

mod buffer;
mod editor;
mod file_io;
mod input;
mod ui;

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    // Set up panic handler to ensure terminal cleanup
    setup_panic_handler();

    // Parse CLI arguments
    let args: Vec<String> = env::args().collect();
    let file_path = parse_args(&args)?;

    // Initialize editor state
    let mut state = if let Some(path) = file_path {
        EditorState::from_file(Path::new(&path))
            .with_context(|| format!("Failed to initialize editor with file: {}", path))?
    } else {
        EditorState::new()
    };

    // Initialize terminal and renderer
    enable_raw_mode().context("Failed to enable raw terminal mode")?;
    let mut renderer = Renderer::new().context("Failed to initialize renderer")?;

    // Initialize input handler with default bindings
    let mut input_handler = InputHandler::with_timeout(Duration::from_millis(1000));
    register_default_bindings(&mut input_handler.registry_mut())
        .context("Failed to register default keybindings")?;

    // Load user config if available (after defaults so User priority takes effect)
    if let Some(config_path) = get_config_path() {
        match load_user_keybindings(&mut input_handler.registry_mut(), &config_path) {
            Ok(count) if count > 0 => {
                eprintln!("Loaded {} user keybinding(s) from {}", count, config_path.display());
            }
            Ok(_) => {
                // Config file exists but is empty - silent
            }
            Err(e) => {
                use std::io::ErrorKind;
                // File not found is OK (silent), other errors get warnings
                if let termide::input::config::ConfigError::ReadError { ref source, .. } = e {
                    if source.kind() != ErrorKind::NotFound {
                        eprintln!("Warning: {}", e);
                    }
                } else {
                    // Parse errors and other issues - warn but continue
                    eprintln!("Warning: {}", e);
                }
            }
        }
    }

    // Initialize cursor position
    let mut cursor = Position::origin();

    // Main event loop
    let result = run_event_loop(&mut state, &mut renderer, &mut cursor, &mut input_handler);

    // Clean up terminal
    disable_raw_mode().context("Failed to disable raw terminal mode")?;
    renderer.restore_terminal().context("Failed to restore terminal")?;

    result
}

/// Parse command-line arguments
///
/// Returns the file path if provided, or None for a new empty buffer.
///
/// # Errors
///
/// Returns an error with usage information if invalid arguments are provided.
fn parse_args(args: &[String]) -> Result<Option<String>> {
    match args.len() {
        1 => {
            // No arguments - new empty buffer
            Ok(None)
        }
        2 => {
            // One argument - file path
            Ok(Some(args[1].clone()))
        }
        _ => {
            // Too many arguments
            anyhow::bail!(
                "Usage: {} [file_path]\n\nArguments:\n  file_path    Optional path to file to open or create",
                args[0]
            );
        }
    }
}

/// Main event loop: read input → process → render
///
/// This loop runs until the user quits the editor.
fn run_event_loop(
    state: &mut EditorState,
    renderer: &mut Renderer,
    cursor: &mut Position,
    input_handler: &mut InputHandler,
) -> Result<()> {
    loop {
        // Render current state
        renderer.render(state, *cursor)?;

        // Check if we should quit
        if state.should_quit() {
            break;
        }

        // Check for sequence buffer timeout
        input_handler.check_timeout();

        // Read input event with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                process_key_event(state, cursor, key_event, input_handler)?;
            }
        }
    }

    Ok(())
}

/// Process a key event by mapping it to a command and executing it
fn process_key_event(
    state: &mut EditorState,
    cursor: &mut Position,
    key_event: KeyEvent,
    input_handler: &mut InputHandler,
) -> Result<()> {
    // Process key event through new input handler
    let result = input_handler.process_key_event(key_event, state.mode());

    match result {
        MatchResult::Matched(cmd) => {
            // Complete match - execute the command
            execute_command(state, cursor, cmd, input_handler)?;
        }
        MatchResult::Partial => {
            // Partial match - wait for next key
            // Do nothing, buffer is preserved automatically
        }
        MatchResult::NoMatch => {
            // No match - fall back to default behavior based on mode
            handle_no_match(state, cursor, key_event)?;
        }
    }

    Ok(())
}

/// Handles keys that don't match any binding (fallback behavior)
///
/// This implements mode-specific default behavior for unmatched keys:
/// - Insert mode: Insert printable characters
/// - Prompt mode: Insert printable characters into prompt
/// - Normal mode: Ignore
fn handle_no_match(
    state: &mut EditorState,
    cursor: &mut Position,
    key_event: KeyEvent,
) -> Result<()> {
    use crossterm::event::{KeyCode, KeyModifiers};

    match state.mode() {
        EditorMode::Insert => {
            // Insert printable characters (only if no special modifiers except SHIFT)
            if let KeyCode::Char(c) = key_event.code {
                if !key_event.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) {
                    // Handle char insertion directly
                    state.handle_char_insert(c, *cursor);
                    // Move cursor after insertion
                    if c == '\n' {
                        cursor.line += 1;
                        cursor.column = 0;
                    } else {
                        cursor.column += 1;
                    }
                    state.clear_status_message();
                }
            }
        }
        EditorMode::Prompt => {
            // Insert printable characters into prompt (only if no special modifiers except SHIFT)
            if let KeyCode::Char(c) = key_event.code {
                if !key_event.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) {
                    state.prompt_insert_char(c);
                }
            }
        }
        EditorMode::Normal => {
            // In Normal mode, ignore unmatched keys
        }
    }

    Ok(())
}

/// Execute an editor command, updating state and cursor position
fn execute_command(
    state: &mut EditorState,
    cursor: &mut Position,
    command: EditorCommand,
    input_handler: &mut InputHandler,
) -> Result<()> {
    match command {
        EditorCommand::InsertChar(ch) => {
            state.handle_char_insert(ch, *cursor);
            // Move cursor after insertion
            if ch == '\n' {
                // Newline: move to start of next line
                cursor.line += 1;
                cursor.column = 0;
            } else {
                // Regular character: move right
                cursor.column += 1;
            }
            // Clear status message on typing
            state.clear_status_message();
        }
        EditorCommand::DeleteChar => {
            if cursor.column > 0 {
                // Delete character before cursor (same line)
                cursor.column -= 1;
                state.handle_char_delete(*cursor);
            } else if cursor.line > 0 {
                // At start of line - join with previous line
                let prev_line_len = state
                    .buffer()
                    .get_line(cursor.line - 1)
                    .map(|line| {
                        // Remove trailing newline if present
                        if line.ends_with('\n') {
                            line.len() - 1
                        } else {
                            line.len()
                        }
                    })
                    .unwrap_or(0);

                cursor.line -= 1;
                cursor.column = prev_line_len;
                state.handle_char_delete(*cursor);
            }
            // Clear status message on editing
            state.clear_status_message();
        }
        EditorCommand::DeleteForward => {
            // Delete character at cursor position (not before)
            state.buffer_mut().delete_forward(*cursor);
            // Cursor stays in same position after forward delete
            // (character after cursor is removed, cursor doesn't move)
            state.clear_status_message();
        }
        EditorCommand::MoveCursor(direction) => {
            move_cursor(cursor, direction, state);
        }
        EditorCommand::MoveToLineStart => {
            *cursor = state.buffer().get_line_start(*cursor);
        }
        EditorCommand::MoveToLineEnd => {
            *cursor = state.buffer().get_line_end(*cursor);
        }
        EditorCommand::PageUp => {
            // Use a reasonable default viewport height (e.g., 20 lines)
            // In the future, this should come from the renderer/terminal height
            const DEFAULT_VIEWPORT_HEIGHT: usize = 20;
            *cursor = state.buffer().page_up(*cursor, DEFAULT_VIEWPORT_HEIGHT);
        }
        EditorCommand::PageDown => {
            // Use a reasonable default viewport height (e.g., 20 lines)
            // In the future, this should come from the renderer/terminal height
            const DEFAULT_VIEWPORT_HEIGHT: usize = 20;
            *cursor = state.buffer().page_down(*cursor, DEFAULT_VIEWPORT_HEIGHT);
        }
        EditorCommand::InsertTab => {
            // Insert a tab character
            // In the future, this should respect editor config (tabs vs spaces, tab width)
            state.handle_char_insert('\t', *cursor);
            cursor.column += 1;
            state.clear_status_message();
        }
        EditorCommand::Save => {
            match state.save() {
                Ok(_saved) => {
                    // Status message already set by save() if file was saved
                    // If not saved, we're now in prompt mode
                }
                Err(e) => {
                    state.set_status_message(format!("Error: {:#}", e));
                }
            }
        }
        EditorCommand::Quit => {
            state.request_quit();
        }
        EditorCommand::ChangeMode(mode) => {
            state.set_mode(mode);
            // Clear sequence buffer on mode change
            input_handler.on_mode_change();
            // Clear status message when changing modes
            state.clear_status_message();
        }
        EditorCommand::PromptInsertChar(ch) => {
            state.prompt_insert_char(ch);
        }
        EditorCommand::PromptDeleteChar => {
            state.prompt_delete_char();
        }
        EditorCommand::AcceptPrompt => {
            let filename = state.accept_prompt();
            // Clear sequence buffer on mode change (Prompt -> previous mode)
            input_handler.on_mode_change();
            if !filename.is_empty() {
                // Save the file with the given filename
                match state.save_as(Path::new(&filename)) {
                    Ok(()) => {
                        // Status message already set by save_as()
                    }
                    Err(e) => {
                        state.set_status_message(format!("Error: {:#}", e));
                    }
                }
            } else {
                state.set_status_message("Error: Filename cannot be empty. Press Esc to cancel.".to_string());
            }
        }
        EditorCommand::CancelPrompt => {
            state.cancel_prompt();
            // Clear sequence buffer on mode change (Prompt -> previous mode)
            input_handler.on_mode_change();
            state.set_status_message("Info: Save cancelled".to_string());
        }
        // Selection commands (placeholder - not yet implemented)
        EditorCommand::SelectLeft
        | EditorCommand::SelectRight
        | EditorCommand::SelectUp
        | EditorCommand::SelectDown
        | EditorCommand::SelectLineStart
        | EditorCommand::SelectLineEnd
        | EditorCommand::SelectAll => {
            state.set_status_message(format!(
                "Command '{:?}' is not yet implemented. This feature is planned for a future release.",
                command
            ));
        }
        // Clipboard commands (placeholder - not yet implemented)
        EditorCommand::Copy | EditorCommand::Cut | EditorCommand::Paste => {
            state.set_status_message(format!(
                "Command '{:?}' is not yet implemented. This feature is planned for a future release.",
                command
            ));
        }
        // Plugin commands
        EditorCommand::PluginCommand {
            plugin_name,
            command_name,
        } => {
            // Plugin system not yet initialized - provide helpful error message
            state.set_status_message(format!(
                "Plugin command '{}.{}' cannot be executed: plugin system not yet initialized. \
                 Plugin support is planned for a future release.",
                plugin_name, command_name
            ));
        }
    }

    Ok(())
}

/// Move cursor in the specified direction, respecting buffer boundaries
fn move_cursor(cursor: &mut Position, direction: Direction, state: &EditorState) {
    let buffer = state.buffer();

    match direction {
        Direction::Up => {
            if cursor.line > 0 {
                cursor.line -= 1;
                // Clamp column to line length
                cursor.column = clamp_column_to_line(cursor.line, cursor.column, buffer);
            }
        }
        Direction::Down => {
            if cursor.line + 1 < buffer.line_count() {
                cursor.line += 1;
                // Clamp column to line length
                cursor.column = clamp_column_to_line(cursor.line, cursor.column, buffer);
            }
        }
        Direction::Left => {
            if cursor.column > 0 {
                cursor.column -= 1;
            } else if cursor.line > 0 {
                // Move to end of previous line
                cursor.line -= 1;
                cursor.column = buffer
                    .get_line(cursor.line)
                    .map(|line| {
                        if line.ends_with('\n') {
                            line.len().saturating_sub(1)
                        } else {
                            line.len()
                        }
                    })
                    .unwrap_or(0);
            }
        }
        Direction::Right => {
            let line_len = buffer
                .get_line(cursor.line)
                .map(|line| {
                    if line.ends_with('\n') {
                        line.len().saturating_sub(1)
                    } else {
                        line.len()
                    }
                })
                .unwrap_or(0);

            if cursor.column < line_len {
                cursor.column += 1;
            } else if cursor.line + 1 < buffer.line_count() {
                // Move to start of next line
                cursor.line += 1;
                cursor.column = 0;
            }
        }
    }
}

/// Clamp column to the length of the current line
fn clamp_column_to_line(line: usize, column: usize, buffer: &termide::buffer::Buffer) -> usize {
    buffer
        .get_line(line)
        .map(|line_content| {
            let len = if line_content.ends_with('\n') {
                line_content.len().saturating_sub(1)
            } else {
                line_content.len()
            };
            column.min(len)
        })
        .unwrap_or(0)
}

/// Set up panic handler to ensure terminal is restored even on panic
fn setup_panic_handler() {
    let original_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal
        let _ = disable_raw_mode();
        let _ = crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen);

        // Call the original panic hook
        original_hook(panic_info);
    }));
}
