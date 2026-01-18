//! Terminal renderer for the editor using Ratatui

use std::io::{self, Stdout};

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::{buffer::Position, editor::EditorState};
use super::Theme;

/// Renders a single frame (standalone function to avoid borrow checker issues)
fn render_frame_impl(
    frame: &mut Frame,
    state: &EditorState,
    cursor_pos: Position,
    scroll_offset: usize,
    theme: &Theme,
) {
    use crate::editor::EditorMode;

    let size = frame.area();

    // Split the terminal into text area and status area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),     // Text area
            Constraint::Length(2),  // Status bar (2 lines: bar + message)
        ])
        .split(size);

    // Render text area
    render_text_area(frame, chunks[0], state, scroll_offset);

    // Render status bar
    render_status_bar(frame, chunks[1], state, cursor_pos, theme);

    // Set cursor position
    if state.mode() == EditorMode::Prompt {
        // In prompt mode, cursor is in the prompt input
        let prompt_cursor_x = chunks[1].x + state.prompt_message().len() as u16 + state.prompt_input().len() as u16;
        let prompt_cursor_y = chunks[1].y + 1; // Second line of status area
        frame.set_cursor_position((prompt_cursor_x, prompt_cursor_y));
    } else {
        // Normal/Insert mode, cursor is in the text area
        let cursor_screen_pos = calculate_cursor_screen_position(cursor_pos, chunks[0], scroll_offset);
        if let Some((x, y)) = cursor_screen_pos {
            frame.set_cursor_position((x, y));
        }
    }
}

/// Renders the text area with buffer content
fn render_text_area(
    frame: &mut Frame,
    area: Rect,
    state: &EditorState,
    scroll_offset: usize,
) {
    let buffer = state.buffer();
    let line_count = buffer.line_count();

    // Calculate visible lines
    let visible_height = area.height as usize;
    let start_line = scroll_offset;
    let end_line = (start_line + visible_height).min(line_count);

    // Collect visible lines
    let mut lines = Vec::new();
    for line_idx in start_line..end_line {
        if let Some(line_content) = buffer.get_line(line_idx) {
            // Remove trailing newline for display
            let display_content = line_content.trim_end_matches('\n');
            lines.push(Line::from(display_content.to_string()));
        }
    }

    // Create paragraph widget
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Renders the status bar and status message
fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    state: &EditorState,
    cursor_pos: Position,
    theme: &Theme,
) {
    use crate::editor::EditorMode;

    // Split status area into bar and message
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Status bar
            Constraint::Length(1), // Status message or prompt
        ])
        .split(area);

    // Build status bar content
    let filename = state
        .buffer()
        .file_path()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("[No Name]");

    let dirty_indicator = if state.buffer().is_dirty() { " *" } else { "" };
    let mode_str = state.mode().to_string();
    let position_str = format!("{}:{}", cursor_pos.line + 1, cursor_pos.column + 1);

    let status_line = format!(
        " {}{} | {} | {}",
        filename, dirty_indicator, mode_str, position_str
    );

    let status_bar = Paragraph::new(status_line).style(
        Style::default()
            .bg(theme.status_bar_bg)
            .fg(theme.status_bar_fg)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_widget(status_bar, chunks[0]);

    // Render prompt or status message
    if state.mode() == EditorMode::Prompt {
        // Display prompt input
        let prompt_text = format!("{}{}", state.prompt_message(), state.prompt_input());
        let prompt_widget = Paragraph::new(prompt_text).style(
            Style::default()
                .fg(theme.prompt_fg)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(prompt_widget, chunks[1]);
    } else if let Some(message) = state.status_message() {
        // Render status message with appropriate color based on type
        let message_style = if message.starts_with("Error:") {
            // Errors in error color
            Style::default().fg(theme.error).add_modifier(Modifier::BOLD)
        } else if message.starts_with("Warning:") {
            // Warnings in warning color
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD)
        } else if message.starts_with("Info:") {
            // Informational messages in info color
            Style::default().fg(theme.info)
        } else {
            // Success messages in success color
            Style::default().fg(theme.success)
        };

        let status_message = Paragraph::new(message).style(message_style);
        frame.render_widget(status_message, chunks[1]);
    }
}

/// Calculates the screen position for the cursor
fn calculate_cursor_screen_position(
    cursor_pos: Position,
    text_area: Rect,
    scroll_offset: usize,
) -> Option<(u16, u16)> {
    // Check if cursor is in visible range
    let visible_line = cursor_pos.line.checked_sub(scroll_offset)?;

    if visible_line >= text_area.height as usize {
        return None;
    }

    let x = text_area.x + cursor_pos.column as u16;
    let y = text_area.y + visible_line as u16;

    // Ensure cursor is within area bounds
    if x >= text_area.x + text_area.width || y >= text_area.y + text_area.height {
        return None;
    }

    Some((x, y))
}

/// The terminal renderer managing UI display
///
/// `Renderer` handles all terminal UI rendering using Ratatui, including:
/// - Buffer content display with viewport optimization
/// - Cursor positioning
/// - Status bar with file info and mode
/// - Status messages
/// - Terminal resize handling
/// - Theming support with customizable colors
///
/// # Performance
///
/// The renderer is optimized to:
/// - Only render visible lines (viewport optimization)
/// - Skip frames when state hasn't changed (dirty checking)
/// - Handle large files efficiently
///
/// # Examples
///
/// ```no_run
/// use termide::ui::Renderer;
/// use termide::editor::EditorState;
/// use termide::buffer::Position;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut renderer = Renderer::new()?;
/// let mut state = EditorState::new();
/// let cursor_pos = Position::origin();
///
/// renderer.render(&state, cursor_pos)?;
/// renderer.restore_terminal()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Renderer {
    /// The Ratatui terminal instance
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Viewport scroll offset (top line visible)
    scroll_offset: usize,
    /// Previous frame hash for dirty checking
    last_frame_hash: u64,
    /// Theme for UI styling
    theme: Theme,
}

impl Renderer {
    /// Creates a new renderer and initializes the terminal with the default theme
    ///
    /// This sets up the terminal in raw mode and alternate screen with the
    /// default dark theme.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use termide::ui::Renderer;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut renderer = Renderer::new()?;
    /// // Use renderer...
    /// renderer.restore_terminal()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        Self::with_theme(Theme::default())
    }

    /// Creates a new renderer with a custom theme
    ///
    /// This sets up the terminal in raw mode and alternate screen with
    /// the specified theme.
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme to use for rendering
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use termide::ui::{Renderer, Theme};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let light_theme = Theme::light();
    /// let mut renderer = Renderer::with_theme(light_theme)?;
    /// // Use renderer...
    /// renderer.restore_terminal()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_theme(theme: Theme) -> Result<Self> {
        enable_raw_mode().context("Failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("Failed to create terminal")?;

        Ok(Self {
            terminal,
            scroll_offset: 0,
            last_frame_hash: 0,
            theme,
        })
    }

    /// Restores the terminal to its normal state
    ///
    /// This should be called before the program exits to clean up the terminal.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal restoration fails.
    pub fn restore_terminal(&mut self) -> Result<()> {
        disable_raw_mode().context("Failed to disable raw mode")?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)
            .context("Failed to leave alternate screen")?;
        self.terminal
            .show_cursor()
            .context("Failed to show cursor")?;
        Ok(())
    }

    /// Renders the editor state to the terminal
    ///
    /// This method handles viewport calculation, scrolling, and renders:
    /// - Buffer content (visible lines only)
    /// - Status bar
    /// - Status messages
    /// - Cursor position
    ///
    /// # Performance
    ///
    /// Only visible lines are rendered. Frame is skipped if state hasn't changed
    /// (based on simple hash comparison).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use termide::ui::Renderer;
    /// use termide::editor::EditorState;
    /// use termide::buffer::Position;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut renderer = Renderer::new()?;
    /// let state = EditorState::new();
    /// let cursor = Position::origin();
    ///
    /// renderer.render(&state, cursor)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn render(&mut self, state: &EditorState, cursor_pos: Position) -> Result<()> {
        // Calculate frame hash for dirty checking
        let frame_hash = self.calculate_frame_hash(state, cursor_pos);

        // Skip rendering if nothing changed
        if frame_hash == self.last_frame_hash {
            return Ok(());
        }

        self.last_frame_hash = frame_hash;

        // Adjust scroll offset based on cursor position
        let terminal_height = self.terminal.size()?.height as usize;
        self.adjust_scroll(cursor_pos, terminal_height);

        // Capture scroll_offset and theme before the closure
        let scroll_offset = self.scroll_offset;
        let theme = &self.theme;

        self.terminal
            .draw(|f| {
                render_frame_impl(f, state, cursor_pos, scroll_offset, theme);
            })
            .context("Failed to draw frame")?;

        Ok(())
    }


    /// Adjusts scroll offset to keep cursor visible
    fn adjust_scroll(&mut self, cursor_pos: Position, terminal_height: usize) {
        // Reserve space for status bar (2 lines)
        let visible_height = terminal_height.saturating_sub(2);

        if visible_height == 0 {
            return;
        }

        // Scroll down if cursor is below visible area
        if cursor_pos.line >= self.scroll_offset + visible_height {
            self.scroll_offset = cursor_pos.line.saturating_sub(visible_height - 1);
        }

        // Scroll up if cursor is above visible area
        if cursor_pos.line < self.scroll_offset {
            self.scroll_offset = cursor_pos.line;
        }
    }

    /// Calculates a simple hash of the current frame state for dirty checking
    ///
    /// This is a simple hash based on buffer content length, cursor position,
    /// mode, status message, and prompt state. It's not perfect but good enough
    /// for skipping unchanged frames.
    fn calculate_frame_hash(&self, state: &EditorState, cursor_pos: Position) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hash = 0u64;

        // Include buffer state
        hash ^= state.buffer().len_chars() as u64;
        hash ^= (state.buffer().line_count() as u64) << 16;
        hash ^= if state.buffer().is_dirty() { 1 } else { 0 } << 32;

        // Include cursor position
        hash ^= (cursor_pos.line as u64) << 8;
        hash ^= (cursor_pos.column as u64) << 24;

        // Include mode
        hash ^= match state.mode() {
            crate::editor::EditorMode::Insert => 1,
            crate::editor::EditorMode::Normal => 2,
            crate::editor::EditorMode::Prompt => 3,
        } << 40;

        // Include status message presence
        hash ^= if state.status_message().is_some() {
            3
        } else {
            0
        } << 48;

        // Include scroll offset
        hash ^= (self.scroll_offset as u64) << 56;

        // Include prompt input if in prompt mode
        if state.mode() == crate::editor::EditorMode::Prompt {
            let mut hasher = DefaultHasher::new();
            state.prompt_input().hash(&mut hasher);
            state.prompt_message().hash(&mut hasher);
            hash ^= hasher.finish();
        }

        hash
    }

    /// Gets the current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the scroll offset manually
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Forces the next frame to render regardless of dirty checking
    pub fn force_render(&mut self) {
        self.last_frame_hash = 0;
    }

    /// Returns a reference to the current theme
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use termide::ui::Renderer;
    /// use ratatui::style::Color;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let renderer = Renderer::new()?;
    /// let selection_color = renderer.theme().selection;
    /// # Ok(())
    /// # }
    /// ```
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Sets a new theme for rendering
    ///
    /// This replaces the current theme and forces a re-render on the next
    /// render call.
    ///
    /// # Arguments
    ///
    /// * `theme` - The new theme to use
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use termide::ui::{Renderer, Theme};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut renderer = Renderer::new()?;
    /// renderer.set_theme(Theme::light());
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.force_render(); // Force re-render with new theme
    }
}

impl Drop for Renderer {
    /// Ensures terminal is restored when renderer is dropped
    fn drop(&mut self) {
        // Best effort terminal restoration
        let _ = self.restore_terminal();
    }
}
