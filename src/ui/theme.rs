//! Theme system for editor styling
//!
//! This module provides a centralized theme system for the editor with customizable
//! colors for all UI components. The theme supports selection highlighting, status bar
//! styling, and status message colors.
//!
//! # Examples
//!
//! ```
//! use termide::ui::Theme;
//!
//! // Use the default dark theme
//! let dark_theme = Theme::default();
//!
//! // Use the light theme
//! let light_theme = Theme::light();
//!
//! // Access selection color
//! let selection_bg = dark_theme.selection;
//! ```
//!
//! # Theme Colors
//!
//! The theme system provides colors for:
//! - **Text selection**: Background color for selected text
//! - **Selection inactive**: Background color when buffer is not focused (future use)
//! - **Status bar**: Background and foreground colors
//! - **Status messages**: Error, warning, info, and success colors
//! - **Prompt**: Input prompt styling

use ratatui::style::Color;

/// Editor theme with color configuration for all UI elements
///
/// `Theme` provides a centralized color scheme for the editor including
/// selection highlighting, status bar, and message colors. It supports
/// both dark and light themes.
///
/// # Examples
///
/// ```
/// use termide::ui::Theme;
/// use ratatui::style::Color;
///
/// let theme = Theme::default();
/// assert_eq!(theme.selection, Color::Rgb(68, 71, 90));
/// ```
///
/// # Selection Colors
///
/// The theme includes two selection-related colors:
/// - `selection`: Used when the buffer is focused (active)
/// - `selection_inactive`: Used when the buffer loses focus (future multi-buffer support)
///
/// Selection colors are designed for good contrast with text while clearly
/// indicating the selected region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    // Selection colors
    /// Background color for selected text when buffer is active
    ///
    /// This color is used to highlight text that has been selected by the user.
    /// It should provide good contrast with the text foreground color while
    /// still being distinguishable from the regular background.
    pub selection: Color,

    /// Background color for selected text when buffer is inactive
    ///
    /// This color is used when the editor has multiple buffers and the selection
    /// is in a non-focused buffer. It's typically a dimmed version of the
    /// active selection color to indicate the buffer doesn't have focus.
    pub selection_inactive: Color,

    // Status bar colors
    /// Background color for the status bar
    pub status_bar_bg: Color,

    /// Foreground (text) color for the status bar
    pub status_bar_fg: Color,

    // Status message colors
    /// Color for error messages (e.g., "Error: File not found")
    pub error: Color,

    /// Color for warning messages (e.g., "Warning: Unsaved changes")
    pub warning: Color,

    /// Color for informational messages (e.g., "Info: File saved")
    pub info: Color,

    /// Color for success messages (e.g., "Saved to file.txt")
    pub success: Color,

    // Prompt colors
    /// Foreground color for prompt text input
    pub prompt_fg: Color,

    // Text colors
    /// Default foreground color for text
    pub text_fg: Color,

    /// Default background color for text area
    pub text_bg: Color,
}

impl Default for Theme {
    /// Creates the default dark theme
    ///
    /// The default theme uses dark colors suitable for most terminal environments.
    /// Selection colors are designed for good contrast and accessibility.
    ///
    /// # Color Choices
    ///
    /// - **Selection**: Dark blue (`#44475a`) - matches Dracula theme palette,
    ///   provides good contrast with white/light text
    /// - **Selection Inactive**: Dimmed gray (`#3b3b3b`) - clearly indicates
    ///   non-focused selection while remaining visible
    /// - **Status Bar**: Dark gray background with white text
    /// - **Messages**: Standard semantic colors (red for errors, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::ui::Theme;
    ///
    /// let theme = Theme::default();
    /// // Use theme.selection for active selection highlighting
    /// ```
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Creates a dark theme (same as default)
    ///
    /// The dark theme is optimized for dark terminal backgrounds and provides
    /// high contrast selection colors that work well with light text.
    ///
    /// # Selection Colors
    ///
    /// - **Active**: `#44475a` (Dracula palette blue-gray) - provides excellent
    ///   contrast with white/light foreground text while clearly indicating selection
    /// - **Inactive**: `#3b3b3b` (dimmed gray) - 50% brightness reduction from
    ///   active selection, still visible but clearly indicates non-focus
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::ui::Theme;
    /// use ratatui::style::Color;
    ///
    /// let theme = Theme::dark();
    /// assert_eq!(theme.selection, Color::Rgb(68, 71, 90));
    /// ```
    pub fn dark() -> Self {
        Self {
            // Selection: Dracula palette dark blue - good contrast with light text
            // RGB(68, 71, 90) = #44475a
            selection: Color::Rgb(68, 71, 90),
            // Selection inactive: Dimmed gray for unfocused buffers
            // RGB(59, 59, 59) = #3b3b3b
            selection_inactive: Color::Rgb(59, 59, 59),

            // Status bar: Dark gray background, white text
            status_bar_bg: Color::DarkGray,
            status_bar_fg: Color::White,

            // Message colors
            error: Color::Red,
            warning: Color::Red, // Same as error for visibility
            info: Color::Cyan,
            success: Color::Green,

            // Prompt
            prompt_fg: Color::Cyan,

            // Text
            text_fg: Color::Reset, // Use terminal default
            text_bg: Color::Reset, // Use terminal default
        }
    }

    /// Creates a light theme optimized for light terminal backgrounds
    ///
    /// The light theme uses colors that provide good contrast on white or
    /// light-colored terminal backgrounds.
    ///
    /// # Selection Colors
    ///
    /// - **Active**: `#add6ff` (light blue) - standard selection color used by
    ///   many editors (VSCode, Sublime) on light backgrounds
    /// - **Inactive**: `#d3d3d3` (light gray) - subtle but visible selection
    ///   indicator for unfocused buffers
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::ui::Theme;
    /// use ratatui::style::Color;
    ///
    /// let theme = Theme::light();
    /// assert_eq!(theme.selection, Color::Rgb(173, 214, 255));
    /// ```
    pub fn light() -> Self {
        Self {
            // Selection: Light blue - standard light theme selection color
            // RGB(173, 214, 255) = #add6ff - similar to VSCode light theme
            selection: Color::Rgb(173, 214, 255),
            // Selection inactive: Light gray for unfocused buffers
            // RGB(211, 211, 211) = #d3d3d3
            selection_inactive: Color::Rgb(211, 211, 211),

            // Status bar: Gray background (lighter than dark theme)
            status_bar_bg: Color::Gray,
            status_bar_fg: Color::Black,

            // Message colors - darker shades for visibility on light backgrounds
            error: Color::Red,
            warning: Color::Red,
            info: Color::Blue, // Blue instead of cyan for better contrast
            success: Color::Green,

            // Prompt
            prompt_fg: Color::Blue,

            // Text
            text_fg: Color::Reset,
            text_bg: Color::Reset,
        }
    }

    /// Returns the selection color based on whether the buffer is active
    ///
    /// # Arguments
    ///
    /// * `is_active` - Whether the buffer currently has focus
    ///
    /// # Returns
    ///
    /// Returns `selection` if active, otherwise `selection_inactive`
    ///
    /// # Examples
    ///
    /// ```
    /// use termide::ui::Theme;
    ///
    /// let theme = Theme::default();
    ///
    /// // Active buffer uses bright selection color
    /// let active_color = theme.selection_color(true);
    /// assert_eq!(active_color, theme.selection);
    ///
    /// // Inactive buffer uses dimmed selection color
    /// let inactive_color = theme.selection_color(false);
    /// assert_eq!(inactive_color, theme.selection_inactive);
    /// ```
    pub fn selection_color(&self, is_active: bool) -> Color {
        if is_active {
            self.selection
        } else {
            self.selection_inactive
        }
    }
}
