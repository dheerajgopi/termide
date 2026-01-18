//! Unit tests for Theme struct

use crate::ui::Theme;
use ratatui::style::Color;

// ============================================================================
// Theme Creation Tests
// ============================================================================

#[test]
fn test_theme_default_creates_dark_theme() {
    let default_theme = Theme::default();
    let dark_theme = Theme::dark();

    // Default should be the same as dark theme
    assert_eq!(default_theme, dark_theme);
}

#[test]
fn test_theme_dark_creation() {
    let theme = Theme::dark();

    // Verify selection colors
    assert_eq!(theme.selection, Color::Rgb(68, 71, 90));
    assert_eq!(theme.selection_inactive, Color::Rgb(59, 59, 59));

    // Verify status bar colors
    assert_eq!(theme.status_bar_bg, Color::DarkGray);
    assert_eq!(theme.status_bar_fg, Color::White);

    // Verify message colors
    assert_eq!(theme.error, Color::Red);
    assert_eq!(theme.warning, Color::Red);
    assert_eq!(theme.info, Color::Cyan);
    assert_eq!(theme.success, Color::Green);

    // Verify prompt color
    assert_eq!(theme.prompt_fg, Color::Cyan);

    // Verify text colors (use terminal defaults)
    assert_eq!(theme.text_fg, Color::Reset);
    assert_eq!(theme.text_bg, Color::Reset);
}

#[test]
fn test_theme_light_creation() {
    let theme = Theme::light();

    // Verify selection colors
    assert_eq!(theme.selection, Color::Rgb(173, 214, 255));
    assert_eq!(theme.selection_inactive, Color::Rgb(211, 211, 211));

    // Verify status bar colors
    assert_eq!(theme.status_bar_bg, Color::Gray);
    assert_eq!(theme.status_bar_fg, Color::Black);

    // Verify message colors
    assert_eq!(theme.error, Color::Red);
    assert_eq!(theme.warning, Color::Red);
    assert_eq!(theme.info, Color::Blue); // Different from dark theme
    assert_eq!(theme.success, Color::Green);

    // Verify prompt color
    assert_eq!(theme.prompt_fg, Color::Blue); // Different from dark theme

    // Verify text colors (use terminal defaults)
    assert_eq!(theme.text_fg, Color::Reset);
    assert_eq!(theme.text_bg, Color::Reset);
}

#[test]
fn test_theme_dark_and_light_are_different() {
    let dark = Theme::dark();
    let light = Theme::light();

    // Selection colors should differ
    assert_ne!(dark.selection, light.selection);
    assert_ne!(dark.selection_inactive, light.selection_inactive);

    // Status bar colors should differ
    assert_ne!(dark.status_bar_bg, light.status_bar_bg);
    assert_ne!(dark.status_bar_fg, light.status_bar_fg);

    // Info and prompt colors should differ
    assert_ne!(dark.info, light.info);
    assert_ne!(dark.prompt_fg, light.prompt_fg);
}

// ============================================================================
// Selection Color Method Tests
// ============================================================================

#[test]
fn test_selection_color_active() {
    let theme = Theme::dark();
    let color = theme.selection_color(true);
    assert_eq!(color, theme.selection);
}

#[test]
fn test_selection_color_inactive() {
    let theme = Theme::dark();
    let color = theme.selection_color(false);
    assert_eq!(color, theme.selection_inactive);
}

#[test]
fn test_selection_color_light_theme_active() {
    let theme = Theme::light();
    let color = theme.selection_color(true);
    assert_eq!(color, Color::Rgb(173, 214, 255));
}

#[test]
fn test_selection_color_light_theme_inactive() {
    let theme = Theme::light();
    let color = theme.selection_color(false);
    assert_eq!(color, Color::Rgb(211, 211, 211));
}

// ============================================================================
// Theme Trait Implementation Tests
// ============================================================================

#[test]
fn test_theme_clone() {
    let theme = Theme::dark();
    let cloned = theme.clone();
    assert_eq!(theme, cloned);
}

#[test]
fn test_theme_debug() {
    let theme = Theme::dark();
    let debug_str = format!("{:?}", theme);
    assert!(debug_str.contains("Theme"));
    assert!(debug_str.contains("selection"));
}

#[test]
fn test_theme_partial_eq() {
    let theme1 = Theme::dark();
    let theme2 = Theme::dark();
    let theme3 = Theme::light();

    assert_eq!(theme1, theme2);
    assert_ne!(theme1, theme3);
}

// ============================================================================
// Color Value Tests
// ============================================================================

#[test]
fn test_dark_theme_selection_rgb_values() {
    let theme = Theme::dark();

    // Verify exact RGB values for selection (Dracula palette)
    if let Color::Rgb(r, g, b) = theme.selection {
        assert_eq!(r, 68);
        assert_eq!(g, 71);
        assert_eq!(b, 90);
    } else {
        panic!("Expected RGB color for selection");
    }
}

#[test]
fn test_light_theme_selection_rgb_values() {
    let theme = Theme::light();

    // Verify exact RGB values for selection (light blue)
    if let Color::Rgb(r, g, b) = theme.selection {
        assert_eq!(r, 173);
        assert_eq!(g, 214);
        assert_eq!(b, 255);
    } else {
        panic!("Expected RGB color for selection");
    }
}

#[test]
fn test_dark_theme_inactive_selection_rgb_values() {
    let theme = Theme::dark();

    // Verify exact RGB values for inactive selection
    if let Color::Rgb(r, g, b) = theme.selection_inactive {
        assert_eq!(r, 59);
        assert_eq!(g, 59);
        assert_eq!(b, 59);
    } else {
        panic!("Expected RGB color for selection_inactive");
    }
}

#[test]
fn test_light_theme_inactive_selection_rgb_values() {
    let theme = Theme::light();

    // Verify exact RGB values for inactive selection (light gray)
    if let Color::Rgb(r, g, b) = theme.selection_inactive {
        assert_eq!(r, 211);
        assert_eq!(g, 211);
        assert_eq!(b, 211);
    } else {
        panic!("Expected RGB color for selection_inactive");
    }
}

// ============================================================================
// Edge Case and Boundary Tests
// ============================================================================

#[test]
fn test_multiple_theme_instances_independent() {
    let theme1 = Theme::dark();
    let theme2 = Theme::dark();
    let _theme3 = Theme::light();

    // Multiple dark themes should be equal
    assert_eq!(theme1, theme2);
}

#[test]
fn test_theme_default_is_usable() {
    // Ensure default theme can be used without panicking
    let theme = Theme::default();

    // All colors should be valid
    let _ = theme.selection;
    let _ = theme.selection_inactive;
    let _ = theme.status_bar_bg;
    let _ = theme.status_bar_fg;
    let _ = theme.error;
    let _ = theme.warning;
    let _ = theme.info;
    let _ = theme.success;
    let _ = theme.prompt_fg;
    let _ = theme.text_fg;
    let _ = theme.text_bg;
}

#[test]
fn test_selection_color_method_consistency() {
    let theme = Theme::dark();

    // Calling with same value should return same result
    assert_eq!(theme.selection_color(true), theme.selection_color(true));
    assert_eq!(theme.selection_color(false), theme.selection_color(false));

    // Calling with different values should return different results
    assert_ne!(theme.selection_color(true), theme.selection_color(false));
}

// ============================================================================
// Integration-style Tests
// ============================================================================

#[test]
fn test_theme_all_fields_accessible() {
    let dark = Theme::dark();
    let light = Theme::light();

    // Verify all fields can be accessed on both themes
    let themes = [dark, light];
    for theme in themes {
        // This ensures all fields are publicly accessible
        let _ = theme.selection;
        let _ = theme.selection_inactive;
        let _ = theme.status_bar_bg;
        let _ = theme.status_bar_fg;
        let _ = theme.error;
        let _ = theme.warning;
        let _ = theme.info;
        let _ = theme.success;
        let _ = theme.prompt_fg;
        let _ = theme.text_fg;
        let _ = theme.text_bg;
    }
}

#[test]
fn test_theme_memory_layout() {
    // Theme should be a reasonable size
    // This is a sanity check to ensure we don't accidentally bloat the struct
    let size = std::mem::size_of::<Theme>();
    // Color enum in ratatui is typically small, so Theme should be compact
    // Each Color is typically 4 bytes (enum variant + data)
    // With 11 Color fields, we expect ~44 bytes plus padding
    assert!(size < 200, "Theme struct is unexpectedly large: {} bytes", size);
}
