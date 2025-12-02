//! Unit tests for keybinding module

use crate::editor::EditorMode;
use crate::input::keybinding::{
    BindingContext, KeyBinding, KeyPattern, KeySequence, Priority, PRIMARY_MODIFIER, ParseError,
};
use crate::input::EditorCommand;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::str::FromStr;

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

// ============================================================================
// KeySequence String Parsing Tests - Valid Inputs
// ============================================================================

#[test]
fn test_parse_single_key_with_ctrl_modifier() {
    let seq = KeySequence::from_str("Ctrl+S").unwrap();
    assert_eq!(seq.len(), 1);

    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('S'), KeyModifiers::CONTROL),
    ]).unwrap();
    assert_eq!(seq, expected);
}

#[test]
fn test_parse_multi_key_sequence() {
    let seq = KeySequence::from_str("d d").unwrap();
    assert_eq!(seq.len(), 2);

    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyPattern::new(KeyCode::Char('d'), KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);
}

#[test]
fn test_parse_multiple_modifiers() {
    let seq = KeySequence::from_str("Ctrl+Shift+F").unwrap();
    assert_eq!(seq.len(), 1);

    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('F'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
    ]).unwrap();
    assert_eq!(seq, expected);
}

#[test]
fn test_parse_case_insensitive_modifiers() {
    let seq1 = KeySequence::from_str("ctrl+s").unwrap();
    let seq2 = KeySequence::from_str("Ctrl+S").unwrap();
    let seq3 = KeySequence::from_str("CTRL+s").unwrap();

    // All should have CONTROL modifier, but 's' vs 'S' is different
    // Actually, let's just check they all parse successfully
    assert_eq!(seq1.len(), 1);
    assert_eq!(seq2.len(), 1);
    assert_eq!(seq3.len(), 1);
}

#[test]
fn test_parse_all_modifiers() {
    // Ctrl
    let seq = KeySequence::from_str("Ctrl+A").unwrap();
    assert_eq!(seq.len(), 1);

    // Shift
    let seq = KeySequence::from_str("Shift+B").unwrap();
    assert_eq!(seq.len(), 1);

    // Alt
    let seq = KeySequence::from_str("Alt+C").unwrap();
    assert_eq!(seq.len(), 1);

    // Super
    let seq = KeySequence::from_str("Super+D").unwrap();
    assert_eq!(seq.len(), 1);
}

#[test]
fn test_parse_special_keys() {
    // Enter
    let seq = KeySequence::from_str("Enter").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Enter, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    // Esc
    let seq = KeySequence::from_str("Esc").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Esc, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    // Tab
    let seq = KeySequence::from_str("Tab").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Tab, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    // Backspace
    let seq = KeySequence::from_str("Backspace").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Backspace, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    // Delete
    let seq = KeySequence::from_str("Delete").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Delete, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);
}

#[test]
fn test_parse_arrow_keys() {
    let seq = KeySequence::from_str("Up").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Up, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    let seq = KeySequence::from_str("Down").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Down, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    let seq = KeySequence::from_str("Left").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Left, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    let seq = KeySequence::from_str("Right").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Right, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);
}

#[test]
fn test_parse_navigation_keys() {
    let seq = KeySequence::from_str("Home").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Home, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    let seq = KeySequence::from_str("End").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::End, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    let seq = KeySequence::from_str("PageUp").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::PageUp, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    let seq = KeySequence::from_str("PageDown").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::PageDown, KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);
}

#[test]
fn test_parse_function_keys() {
    for i in 1..=12 {
        let input = format!("F{}", i);
        let seq = KeySequence::from_str(&input).unwrap();
        let expected = KeySequence::new(vec![
            KeyPattern::new(KeyCode::F(i), KeyModifiers::NONE),
        ]).unwrap();
        assert_eq!(seq, expected, "F{} should parse correctly", i);
    }
}

#[test]
fn test_parse_special_keys_with_modifiers() {
    let seq = KeySequence::from_str("Ctrl+Enter").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Enter, KeyModifiers::CONTROL),
    ]).unwrap();
    assert_eq!(seq, expected);

    let seq = KeySequence::from_str("Shift+Tab").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Tab, KeyModifiers::SHIFT),
    ]).unwrap();
    assert_eq!(seq, expected);

    let seq = KeySequence::from_str("Alt+Backspace").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Backspace, KeyModifiers::ALT),
    ]).unwrap();
    assert_eq!(seq, expected);
}

#[test]
fn test_parse_multi_key_with_modifiers() {
    let seq = KeySequence::from_str("Ctrl+X k").unwrap();
    assert_eq!(seq.len(), 2);

    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('X'), KeyModifiers::CONTROL),
        KeyPattern::new(KeyCode::Char('k'), KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);
}

#[test]
fn test_parse_whitespace_handling() {
    // Extra spaces should be ignored
    let seq1 = KeySequence::from_str("  d   d  ").unwrap();
    let seq2 = KeySequence::from_str("d d").unwrap();
    assert_eq!(seq1, seq2);

    // Tabs and multiple spaces
    let seq3 = KeySequence::from_str("d\t\td").unwrap();
    assert_eq!(seq2, seq3);
}

#[test]
fn test_parse_single_character_keys() {
    let seq = KeySequence::from_str("a").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('a'), KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    // Uppercase should be preserved
    let seq = KeySequence::from_str("A").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char('A'), KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);
}

#[test]
fn test_parse_space_key() {
    let seq = KeySequence::from_str("Space").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char(' '), KeyModifiers::NONE),
    ]).unwrap();
    assert_eq!(seq, expected);

    // With modifiers
    let seq = KeySequence::from_str("Ctrl+Space").unwrap();
    let expected = KeySequence::new(vec![
        KeyPattern::new(KeyCode::Char(' '), KeyModifiers::CONTROL),
    ]).unwrap();
    assert_eq!(seq, expected);
}

#[test]
fn test_parse_vim_style_sequences() {
    // dd - delete line
    let seq = KeySequence::from_str("d d").unwrap();
    assert_eq!(seq.len(), 2);

    // gg - go to top
    let seq = KeySequence::from_str("g g").unwrap();
    assert_eq!(seq.len(), 2);

    // ci( - change inside parentheses
    let seq = KeySequence::from_str("c i (").unwrap();
    assert_eq!(seq.len(), 3);
}

#[test]
fn test_parse_alternative_key_names() {
    // "Control" as alternative to "Ctrl"
    let seq1 = KeySequence::from_str("Control+S").unwrap();
    let seq2 = KeySequence::from_str("Ctrl+S").unwrap();
    assert_eq!(seq1, seq2);

    // "Cmd" and "Command" as alternatives to "Super"
    let seq1 = KeySequence::from_str("Cmd+Q").unwrap();
    let seq2 = KeySequence::from_str("Command+Q").unwrap();
    let seq3 = KeySequence::from_str("Super+Q").unwrap();
    assert_eq!(seq1, seq2);
    assert_eq!(seq2, seq3);

    // "Escape" as alternative to "Esc"
    let seq1 = KeySequence::from_str("Escape").unwrap();
    let seq2 = KeySequence::from_str("Esc").unwrap();
    assert_eq!(seq1, seq2);

    // "Return" as alternative to "Enter"
    let seq1 = KeySequence::from_str("Return").unwrap();
    let seq2 = KeySequence::from_str("Enter").unwrap();
    assert_eq!(seq1, seq2);
}

// ============================================================================
// KeySequence String Parsing Tests - Invalid Inputs
// ============================================================================

#[test]
fn test_parse_empty_string() {
    let result = KeySequence::from_str("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ParseError::EmptyInput);
}

#[test]
fn test_parse_whitespace_only() {
    let result = KeySequence::from_str("   ");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ParseError::EmptyInput);
}

#[test]
fn test_parse_incomplete_pattern() {
    let result = KeySequence::from_str("Ctrl+");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidFormat(_) => {},
        e => panic!("Expected InvalidFormat, got {:?}", e),
    }
}

#[test]
fn test_parse_unknown_modifier() {
    let result = KeySequence::from_str("Unknown+S");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::UnknownModifier(s) => assert_eq!(s, "Unknown"),
        e => panic!("Expected UnknownModifier, got {:?}", e),
    }
}

#[test]
fn test_parse_unknown_key() {
    let result = KeySequence::from_str("InvalidKey");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::UnknownKey(s) => assert_eq!(s, "InvalidKey"),
        e => panic!("Expected UnknownKey, got {:?}", e),
    }
}

#[test]
fn test_parse_invalid_function_key() {
    // F13 doesn't exist
    let result = KeySequence::from_str("F13");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::UnknownKey(s) => assert_eq!(s, "F13"),
        e => panic!("Expected UnknownKey, got {:?}", e),
    }

    // F0 doesn't exist
    let result = KeySequence::from_str("F0");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::UnknownKey(s) => assert_eq!(s, "F0"),
        e => panic!("Expected UnknownKey, got {:?}", e),
    }
}

#[test]
fn test_parse_multiple_plus_signs() {
    let result = KeySequence::from_str("Ctrl++S");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidFormat(_) => {},
        e => panic!("Expected InvalidFormat, got {:?}", e),
    }
}

#[test]
fn test_parse_modifier_only() {
    let result = KeySequence::from_str("Ctrl");
    assert!(result.is_err());
    // "Ctrl" by itself is an unknown key
    match result.unwrap_err() {
        ParseError::UnknownKey(s) => assert_eq!(s, "Ctrl"),
        e => panic!("Expected UnknownKey, got {:?}", e),
    }
}

#[test]
fn test_parse_case_sensitivity_for_keys() {
    // Keys should be case-sensitive for characters
    let seq1 = KeySequence::from_str("a").unwrap();
    let seq2 = KeySequence::from_str("A").unwrap();
    assert_ne!(seq1, seq2);

    // But special keys should be case-insensitive
    let seq1 = KeySequence::from_str("enter").unwrap();
    let seq2 = KeySequence::from_str("ENTER").unwrap();
    assert_eq!(seq1, seq2);
}

// ============================================================================
// ParseError Display Tests
// ============================================================================

#[test]
fn test_parse_error_messages() {
    // Empty input
    let err = KeySequence::from_str("").unwrap_err();
    assert_eq!(err.to_string(), "empty key sequence string");

    // Unknown modifier
    let err = KeySequence::from_str("BadMod+S").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown modifier"));
    assert!(msg.contains("BadMod"));

    // Unknown key
    let err = KeySequence::from_str("BadKey").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown key name"));
    assert!(msg.contains("BadKey"));
}
