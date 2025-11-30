//! Unit tests for keybinding module

use crate::editor::EditorMode;
use crate::input::keybinding::{
    BindingContext, KeyBinding, KeyPattern, KeySequence, Priority, PRIMARY_MODIFIER,
};
use crate::input::EditorCommand;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Helper to create a KeyEvent from code and modifiers
fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

// ============================================================================
// KeyPattern Tests
// ============================================================================

#[test]
fn test_key_pattern_matches_with_no_modifiers() {
    let pattern = KeyPattern::new(KeyCode::Char('a'), KeyModifiers::NONE);
    let event = key_event(KeyCode::Char('a'), KeyModifiers::NONE);
    assert!(pattern.matches(&event));
}

#[test]
fn test_key_pattern_matches_with_modifiers() {
    let pattern = KeyPattern::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let event = key_event(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(pattern.matches(&event));

    // Multiple modifiers
    let pattern = KeyPattern::new(
        KeyCode::Char('x'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    let event = key_event(
        KeyCode::Char('x'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert!(pattern.matches(&event));
}

#[test]
fn test_key_pattern_exact_modifier_matching() {
    let pattern = KeyPattern::new(KeyCode::Char('s'), KeyModifiers::CONTROL);

    // Extra modifiers should NOT match (exact matching)
    let event = key_event(
        KeyCode::Char('s'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert!(!pattern.matches(&event));

    // Missing modifiers should NOT match
    let event = key_event(KeyCode::Char('s'), KeyModifiers::NONE);
    assert!(!pattern.matches(&event));

    // Wrong key code should NOT match
    let event = key_event(KeyCode::Char('x'), KeyModifiers::CONTROL);
    assert!(!pattern.matches(&event));
}

#[test]
fn test_key_pattern_vim_style_modifier_distinction() {
    // In vim, 'd' and 'D' (Shift+d) are different commands
    let lowercase_d = KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE);
    let uppercase_d = KeyPattern::new(KeyCode::Char('d'), KeyModifiers::SHIFT);

    let event_d = key_event(KeyCode::Char('d'), KeyModifiers::NONE);
    let event_shift_d = key_event(KeyCode::Char('d'), KeyModifiers::SHIFT);

    assert!(lowercase_d.matches(&event_d));
    assert!(!lowercase_d.matches(&event_shift_d));
    assert!(uppercase_d.matches(&event_shift_d));
    assert!(!uppercase_d.matches(&event_d));
}

#[test]
fn test_key_pattern_special_keys() {
    let test_cases = vec![
        (KeyCode::Esc, KeyModifiers::NONE),
        (KeyCode::Enter, KeyModifiers::NONE),
        (KeyCode::Backspace, KeyModifiers::NONE),
        (KeyCode::Up, KeyModifiers::NONE),
        (KeyCode::Right, KeyModifiers::CONTROL),
    ];

    for (code, modifiers) in test_cases {
        let pattern = KeyPattern::new(code, modifiers);
        let event = key_event(code, modifiers);
        assert!(pattern.matches(&event));
    }
}

#[test]
fn test_primary_modifier_constant() {
    // Verify PRIMARY_MODIFIER is correct for the platform
    #[cfg(target_os = "macos")]
    assert_eq!(PRIMARY_MODIFIER, KeyModifiers::SUPER);

    #[cfg(not(target_os = "macos"))]
    assert_eq!(PRIMARY_MODIFIER, KeyModifiers::CONTROL);

    // Verify it can be used in patterns
    let save = KeyPattern::new(KeyCode::Char('s'), PRIMARY_MODIFIER);

    #[cfg(target_os = "macos")]
    {
        let event = key_event(KeyCode::Char('s'), KeyModifiers::SUPER);
        assert!(save.matches(&event));
    }

    #[cfg(not(target_os = "macos"))]
    {
        let event = key_event(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(save.matches(&event));
    }
}

// ============================================================================
// KeySequence Tests
// ============================================================================

#[test]
fn test_key_sequence_new_returns_some_for_valid_input() {
    let seq = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
    ]);
    assert!(seq.is_some());
    assert_eq!(seq.unwrap().len(), 1);

    let seq = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ]);
    assert!(seq.is_some());
    assert_eq!(seq.unwrap().len(), 2);
}

#[test]
fn test_key_sequence_new_returns_none_for_empty() {
    let seq = KeySequence::new(vec![]);
    assert!(seq.is_none());
}

#[test]
fn test_key_sequence_matches_exact_sequence() {
    let seq = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ])
    .expect("valid sequence");

    // Exact match
    let buffer = vec![
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ];
    assert!(seq.matches(&buffer));

    // Too short - no match
    let buffer = vec![KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE)];
    assert!(!seq.matches(&buffer));

    // Too long - no match
    let buffer = vec![
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ];
    assert!(!seq.matches(&buffer));

    // Wrong keys - no match
    let buffer = vec![
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('x'), KeyModifiers::NONE),
    ];
    assert!(!seq.matches(&buffer));

    // Empty buffer - no match
    assert!(!seq.matches(&[]));
}

#[test]
fn test_key_sequence_partial_match_detection() {
    let seq = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ])
    .expect("valid sequence");

    // First key typed - partial match
    let buffer = vec![KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE)];
    assert!(seq.is_partial_match(&buffer));

    // Complete sequence - NOT a partial match
    let buffer = vec![
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ];
    assert!(!seq.is_partial_match(&buffer));

    // Empty buffer - NOT a partial match
    assert!(!seq.is_partial_match(&[]));

    // Wrong prefix - NOT a partial match
    let buffer = vec![KeyPattern::new(KeyCode::Char('x'), KeyModifiers::NONE)];
    assert!(!seq.is_partial_match(&buffer));

    // Too long - NOT a partial match
    let buffer = vec![
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ];
    assert!(!seq.is_partial_match(&buffer));
}

#[test]
fn test_key_sequence_partial_match_multi_step() {
    // Three-key sequence
    let seq = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
    ])
    .expect("valid sequence");

    // First key - partial
    let buffer = vec![KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE)];
    assert!(seq.is_partial_match(&buffer));

    // First two keys - partial
    let buffer = vec![
        KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
    ];
    assert!(seq.is_partial_match(&buffer));

    // All three keys - complete (not partial)
    let buffer = vec![
        KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('g'), KeyModifiers::NONE),
    ];
    assert!(!seq.is_partial_match(&buffer));
    assert!(seq.matches(&buffer));
}

#[test]
fn test_key_sequence_with_modifiers() {
    let seq = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        KeyPattern::new(KeyCode::Char('x'), KeyModifiers::NONE),
    ])
    .expect("valid sequence");

    // Partial match
    let buffer = vec![KeyPattern::new(KeyCode::Char('c'), KeyModifiers::CONTROL)];
    assert!(seq.is_partial_match(&buffer));

    // Complete match
    let buffer = vec![
        KeyPattern::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        KeyPattern::new(KeyCode::Char('x'), KeyModifiers::NONE),
    ];
    assert!(seq.matches(&buffer));

    // Wrong modifiers on first key - no match
    let buffer = vec![KeyPattern::new(KeyCode::Char('c'), KeyModifiers::NONE)];
    assert!(!seq.is_partial_match(&buffer));
}

#[test]
fn test_key_sequence_single_pattern_no_partial_match() {
    let seq = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE),
    ])
    .expect("valid sequence");

    // Single-pattern sequences can never have partial matches
    // (buffer is either empty or complete)
    assert!(!seq.is_partial_match(&[]));

    let buffer = vec![KeyPattern::new(KeyCode::Char('i'), KeyModifiers::NONE)];
    assert!(!seq.is_partial_match(&buffer));
    assert!(seq.matches(&buffer));
}

// ============================================================================
// BindingContext Tests
// ============================================================================

#[test]
fn test_binding_context_global_excludes_prompt() {
    let ctx = BindingContext::Global;

    // Global should be active in Insert and Normal
    assert!(ctx.is_active(EditorMode::Insert));
    assert!(ctx.is_active(EditorMode::Normal));

    // Global should NOT be active in Prompt (Prompt mode should handle its own bindings)
    assert!(!ctx.is_active(EditorMode::Prompt));
}

#[test]
fn test_binding_context_mode_specific() {
    let ctx = BindingContext::Mode(EditorMode::Normal);

    // Should only be active in Normal mode
    assert!(ctx.is_active(EditorMode::Normal));
    assert!(!ctx.is_active(EditorMode::Insert));
    assert!(!ctx.is_active(EditorMode::Prompt));
}

#[test]
fn test_binding_context_multi_mode() {
    let ctx = BindingContext::Modes(vec![EditorMode::Insert, EditorMode::Normal]);

    // Should be active in both Insert and Normal
    assert!(ctx.is_active(EditorMode::Insert));
    assert!(ctx.is_active(EditorMode::Normal));

    // Should NOT be active in Prompt
    assert!(!ctx.is_active(EditorMode::Prompt));
}

#[test]
fn test_binding_context_plugin_without_mode_filter() {
    let ctx = BindingContext::Plugin {
        name: "lsp".to_string(),
        modes: None,
    };

    // Plugin without mode filter should be active in all modes
    assert!(ctx.is_active(EditorMode::Insert));
    assert!(ctx.is_active(EditorMode::Normal));
    assert!(ctx.is_active(EditorMode::Prompt));
}

#[test]
fn test_binding_context_plugin_with_mode_filter() {
    let ctx = BindingContext::Plugin {
        name: "lsp".to_string(),
        modes: Some(vec![EditorMode::Normal, EditorMode::Insert]),
    };

    // Plugin with mode filter should only be active in specified modes
    assert!(ctx.is_active(EditorMode::Insert));
    assert!(ctx.is_active(EditorMode::Normal));
    assert!(!ctx.is_active(EditorMode::Prompt));
}

// ============================================================================
// Priority Tests
// ============================================================================

#[test]
fn test_priority_ordering() {
    // Higher priority values should be greater
    assert!(Priority::User > Priority::Plugin);
    assert!(Priority::Plugin > Priority::Default);
    assert!(Priority::User > Priority::Default);
}

#[test]
fn test_priority_numeric_values() {
    // Verify the exact numeric values
    assert_eq!(Priority::Default as u8, 0);
    assert_eq!(Priority::Plugin as u8, 10);
    assert_eq!(Priority::User as u8, 20);
}

// ============================================================================
// KeyBinding Tests
// ============================================================================

#[test]
fn test_key_binding_accessors() {
    let seq = KeySequence::new(vec![KeyPattern::new(
        KeyCode::Char('i'),
        KeyModifiers::NONE,
    )])
    .expect("valid sequence");

    let binding = KeyBinding::new(
        seq.clone(),
        EditorCommand::ChangeMode(EditorMode::Insert),
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    assert_eq!(binding.sequence(), &seq);
    assert_eq!(
        binding.command(),
        &EditorCommand::ChangeMode(EditorMode::Insert)
    );
    assert_eq!(binding.context(), &BindingContext::Mode(EditorMode::Normal));
    assert_eq!(binding.priority(), Priority::Default);
}

#[test]
fn test_key_binding_global_save() {
    let seq =
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('s'), PRIMARY_MODIFIER)])
            .expect("valid sequence");

    let binding = KeyBinding::new(
        seq,
        EditorCommand::Save,
        BindingContext::Global,
        Priority::Default,
    );

    // Global binding should be active in Insert and Normal
    assert!(binding.context().is_active(EditorMode::Insert));
    assert!(binding.context().is_active(EditorMode::Normal));
    assert!(!binding.context().is_active(EditorMode::Prompt));
}

#[test]
fn test_key_binding_mode_specific() {
    let seq = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ])
    .expect("valid sequence");

    let binding = KeyBinding::new(
        seq,
        EditorCommand::DeleteChar,
        BindingContext::Mode(EditorMode::Normal),
        Priority::Default,
    );

    // Should only be active in Normal mode
    assert!(binding.context().is_active(EditorMode::Normal));
    assert!(!binding.context().is_active(EditorMode::Insert));
}

#[test]
fn test_key_binding_priority_levels() {
    let seq =
        KeySequence::new(vec![KeyPattern::new(KeyCode::Char('s'), PRIMARY_MODIFIER)])
            .expect("valid sequence");

    let default_binding = KeyBinding::new(
        seq.clone(),
        EditorCommand::Save,
        BindingContext::Global,
        Priority::Default,
    );

    let user_binding = KeyBinding::new(
        seq,
        EditorCommand::Save,
        BindingContext::Global,
        Priority::User,
    );

    // User priority should be higher than Default
    assert!(user_binding.priority() > default_binding.priority());
}
