//! Comprehensive integration tests for Keylaut engine, edge cases, and property invariants.

use keylaut::core::config::Config;
use keylaut::core::event::{Key, KeyEvent, Modifiers};
use keylaut::core::state::EngineAction;
use keylaut::core::KeylautEngine;

fn simulate_typing(engine: &mut KeylautEngine, text: &str) -> Vec<EngineAction> {
    let mut actions = Vec::new();
    for c in text.chars() {
        let key = match c {
            ' ' => Key::Space,
            '\n' => Key::Enter,
            '\t' => Key::Tab,
            ch => Key::Char(ch),
        };
        let event = KeyEvent::press(key, Modifiers::NONE);
        actions.push(engine.process_event(event));
    }
    actions
}

// ---------------------------------------------------------------------------
// 1. Basic Replacements
// ---------------------------------------------------------------------------

#[test]
fn test_basic_replacements() {
    let test_cases = vec![("ae ", "ä "), ("oe ", "ö "), ("ue ", "ü "), ("ss ", "ß ")];

    for (input, expected) in test_cases {
        let mut engine = KeylautEngine::new(Config::default());
        let actions = simulate_typing(&mut engine, input);
        assert_eq!(
            actions.last(),
            Some(&EngineAction::Replace {
                backspaces: 2,
                text: expected.to_string(),
            }),
            "Failed on input: {}",
            input
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Capitalization
// ---------------------------------------------------------------------------

#[test]
fn test_capitalization_replacements() {
    let test_cases = vec![
        ("AE ", "Ä "),
        ("OE ", "Ö "),
        ("UE ", "Ü "),
        ("Ae ", "Ä "),
        ("Oe ", "Ö "),
        ("Ue ", "Ü "),
    ];

    for (input, expected) in test_cases {
        let mut engine = KeylautEngine::new(Config::default());
        let actions = simulate_typing(&mut engine, input);
        assert_eq!(
            actions.last(),
            Some(&EngineAction::Replace {
                backspaces: 2,
                text: expected.to_string(),
            }),
            "Failed on input: {}",
            input
        );
    }
}

#[test]
fn test_ss_uppercase_not_default() {
    let mut engine = KeylautEngine::new(Config::default());
    let actions = simulate_typing(&mut engine, "SS ");
    // SS should NOT become ẞ by default
    assert_eq!(actions.last(), Some(&EngineAction::Pass));
}

// ---------------------------------------------------------------------------
// 3. False Positives
// ---------------------------------------------------------------------------

#[test]
fn test_false_positives_untouched() {
    let words = vec![
        "aesthetic ",
        "aerospace ",
        "aerodynamic ",
        "issue ",
        "coefficient ",
        "aes ",
        "aer ",
        "shoe ",
        "shoes ",
        "toe ",
        "toes ",
        "canoe ",
        "blue ",
        "clue ",
        "true ",
        "venue ",
        "argue ",
        "class ",
        "pass ",
        "process ",
        "access ",
        "success ",
        "business ",
    ];

    for word in words {
        let mut engine = KeylautEngine::new(Config::default());
        let actions = simulate_typing(&mut engine, word);
        assert_eq!(
            actions.last(),
            Some(&EngineAction::Pass),
            "False positive triggered for word: {}",
            word
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Word Boundaries
// ---------------------------------------------------------------------------

#[test]
fn test_word_boundaries_fuer_schoen_groesser() {
    let words = vec![("fuer", "für"), ("schoen", "schön"), ("groesser", "größer")];

    let delimiters = vec![
        ('.', '.'),
        (',', ','),
        ('!', '!'),
        ('?', '?'),
        (':', ':'),
        (';', ';'),
        (' ', ' '),
        ('\n', '\n'),
    ];

    for (word, expected_root) in &words {
        for (delim, expected_delim) in &delimiters {
            let mut engine = KeylautEngine::new(Config::default());
            let input = format!("{}{}", word, delim);
            let actions = simulate_typing(&mut engine, &input);

            let expected_text = format!("{}{}", expected_root, expected_delim);
            assert_eq!(
                actions.last(),
                Some(&EngineAction::Replace {
                    backspaces: word.chars().count(),
                    text: expected_text.clone(),
                }),
                "Failed on input: {}",
                input
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Natural Sentences from Spec
// ---------------------------------------------------------------------------

#[test]
fn test_spec_full_sentences() {
    // 1. "Das ist schoen." -> "Das ist schön."
    {
        let mut engine = KeylautEngine::new(Config::default());
        let actions = simulate_typing(&mut engine, "Das ist schoen.");
        assert_eq!(
            actions.last(),
            Some(&EngineAction::Replace {
                backspaces: 6,
                text: "schön.".to_string(),
            })
        );
    }

    // 2. "Ich bin fuer dich da." -> "Ich bin für dich da."
    {
        let mut engine = KeylautEngine::new(Config::default());
        simulate_typing(&mut engine, "Ich bin fuer ");
        // fuer was transformed
        let actions = simulate_typing(&mut engine, "dich da.");
        assert_eq!(actions.last(), Some(&EngineAction::Pass));
    }

    // 3. "Ich gehe spaeter nach Hause." -> "Ich gehe später nach Hause."
    {
        let mut engine = KeylautEngine::new(Config::default());
        let actions = simulate_typing(&mut engine, "spaeter ");
        assert_eq!(
            actions.last(),
            Some(&EngineAction::Replace {
                backspaces: 7,
                text: "später ".to_string(),
            })
        );
    }
}

// ---------------------------------------------------------------------------
// 6. Editing and Navigation Keys
// ---------------------------------------------------------------------------

#[test]
fn test_backspace_editing() {
    let mut engine = KeylautEngine::new(Config::default());

    // Type "fue"
    simulate_typing(&mut engine, "fue");
    assert_eq!(engine.current_word(), "fue");

    // Backspace: removes 'e' -> "fu"
    engine.process_event(KeyEvent::press(Key::Backspace, Modifiers::NONE));
    assert_eq!(engine.current_word(), "fu");

    // Type "n" -> "fun"
    simulate_typing(&mut engine, "n ");
    // "fun " contains no mappings
    assert_eq!(engine.current_word(), "");
}

#[test]
fn test_escape_cancels_candidate() {
    let mut engine = KeylautEngine::new(Config::default());
    simulate_typing(&mut engine, "ae");
    assert_eq!(engine.current_word(), "ae");

    // Escape clears
    let action = engine.process_event(KeyEvent::press(Key::Escape, Modifiers::NONE));
    assert_eq!(action, EngineAction::Pass);
    assert_eq!(engine.current_word(), "");

    // Subsequent space does not trigger replacement
    let action = engine.process_event(KeyEvent::press(Key::Space, Modifiers::NONE));
    assert_eq!(action, EngineAction::Pass);
}

#[test]
fn test_navigation_keys_cancel() {
    let nav_keys = vec![
        Key::Left,
        Key::Right,
        Key::Up,
        Key::Down,
        Key::Home,
        Key::End,
        Key::PageUp,
        Key::PageDown,
        Key::Delete,
    ];

    for nav in nav_keys {
        let mut engine = KeylautEngine::new(Config::default());
        simulate_typing(&mut engine, "schoe");
        assert_eq!(engine.current_word(), "schoe");

        engine.process_event(KeyEvent::press(nav, Modifiers::NONE));
        assert_eq!(engine.current_word(), "");
    }
}

// ---------------------------------------------------------------------------
// 7. Modifiers
// ---------------------------------------------------------------------------

#[test]
fn test_command_modifiers_ignored() {
    let mut engine = KeylautEngine::new(Config::default());
    simulate_typing(&mut engine, "schoen");

    // User hits Cmd+C or Ctrl+C
    let action = engine.process_event(KeyEvent::press(
        Key::Char('c'),
        Modifiers {
            ctrl: true,
            shift: false,
            alt: false,
            meta: false,
        },
    ));

    // Must pass through and reset
    assert_eq!(action, EngineAction::Pass);
    assert_eq!(engine.current_word(), "");
}

// ---------------------------------------------------------------------------
// 8. Property-Based and Invariant Testing
// ---------------------------------------------------------------------------

#[test]
fn test_property_invariants() {
    let mut engine = KeylautEngine::new(Config::default());

    // Invariant 1: Unrelated plain characters without umlaut pairs must never be modified
    let words = vec!["hallo ", "welt ", "test ", "keyboard ", "rust ", "quick "];
    for w in words {
        let actions = simulate_typing(&mut engine, w);
        for a in actions {
            assert!(
                a.is_pass(),
                "Unrelated word '{}' produced non-pass action {:?}",
                w,
                a
            );
        }
    }

    // Invariant 2: Replacements only ever emit configured characters (ä, ö, ü, ß, Ä, Ö, Ü)
    let valid_umlauts = ['ä', 'ö', 'ü', 'ß', 'Ä', 'Ö', 'Ü'];
    let inputs = vec![
        "ae ",
        "oe ",
        "ue ",
        "ss ",
        "Ae ",
        "Oe ",
        "Ue ",
        "AE ",
        "OE ",
        "UE ",
        "schoen! ",
        "fuer? ",
        "groesser. ",
        "spaeter; ",
    ];

    for input in inputs {
        let mut e = KeylautEngine::new(Config::default());
        let actions = simulate_typing(&mut e, input);
        for a in actions {
            if let EngineAction::Replace { text, .. } = a {
                let contains_umlaut = text.chars().any(|c| valid_umlauts.contains(&c));
                assert!(
                    contains_umlaut,
                    "Replacement text '{}' did not contain any valid umlauts",
                    text
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 9. Rapid Input Fuzzing
// ---------------------------------------------------------------------------

#[test]
fn test_fuzz_random_event_sequences() {
    let mut engine = KeylautEngine::new(Config::default());
    let corpus = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,!?;:\n\t";

    // Pseudo-random deterministic sequence
    let mut seed: u64 = 0x12345678;
    for _ in 0..10_000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = ((seed >> 32) as usize) % corpus.len();
        let ch = corpus.chars().nth(idx).unwrap();

        let key = match ch {
            ' ' => Key::Space,
            '\n' => Key::Enter,
            '\t' => Key::Tab,
            c => Key::Char(c),
        };

        // Engine must never panic or enter an illegal state
        let action = engine.process_event(KeyEvent::press(key, Modifiers::NONE));
        match action {
            EngineAction::Pass | EngineAction::Suppress => {}
            EngineAction::Replace { backspaces, text } => {
                assert!(backspaces > 0);
                assert!(!text.is_empty());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 10. Hold-to-Bypass Feature Tests
// ---------------------------------------------------------------------------

#[test]
fn test_hold_alt_on_delimiter_bypasses_replacement() {
    let mut engine = KeylautEngine::new(Config::default());

    // Type "ae" without modifiers
    engine.process_event(KeyEvent::char_press('a', Modifiers::NONE));
    engine.process_event(KeyEvent::char_press('e', Modifiers::NONE));

    // Space pressed with Alt held (Option on Mac)
    let alt_space = KeyEvent::press(
        Key::Space,
        Modifiers {
            alt: true,
            ..Modifiers::NONE
        },
    );
    let action = engine.process_event(alt_space);

    // Must NOT replace with "ä " - must pass through!
    assert_eq!(action, EngineAction::Pass);
    assert_eq!(engine.current_word(), "");
}

#[test]
fn test_hold_alt_during_typing_bypasses_replacement() {
    let mut engine = KeylautEngine::new(Config::default());

    let alt_mod = Modifiers {
        alt: true,
        ..Modifiers::NONE
    };

    // Type "f" (normal), "u" (normal), "e" (with Alt held), "r" (normal)
    engine.process_event(KeyEvent::char_press('f', Modifiers::NONE));
    engine.process_event(KeyEvent::char_press('u', Modifiers::NONE));
    engine.process_event(KeyEvent::char_press('e', alt_mod));
    engine.process_event(KeyEvent::char_press('r', Modifiers::NONE));

    // Delimiter typed normally without Alt
    let action = engine.process_event(KeyEvent::press(Key::Space, Modifiers::NONE));

    // Must NOT replace "fuer" with "für", because Alt was held during word typing!
    assert_eq!(action, EngineAction::Pass);
    assert_eq!(engine.current_word(), "");
}

#[test]
fn test_hold_alt_does_not_affect_next_word() {
    let mut engine = KeylautEngine::new(Config::default());

    // Word 1: "ae" + Alt-Space -> bypassed, not replaced
    engine.process_event(KeyEvent::char_press('a', Modifiers::NONE));
    engine.process_event(KeyEvent::char_press('e', Modifiers::NONE));
    let action1 = engine.process_event(KeyEvent::press(
        Key::Space,
        Modifiers {
            alt: true,
            ..Modifiers::NONE
        },
    ));
    assert_eq!(action1, EngineAction::Pass);

    // Word 2: "oe" + normal Space -> SHOULD be replaced with "ö "
    engine.process_event(KeyEvent::char_press('o', Modifiers::NONE));
    engine.process_event(KeyEvent::char_press('e', Modifiers::NONE));
    let action2 = engine.process_event(KeyEvent::press(Key::Space, Modifiers::NONE));
    assert_eq!(
        action2,
        EngineAction::Replace {
            backspaces: 2,
            text: "ö ".to_string()
        }
    );
}

#[test]
fn test_configurable_bypass_key_ctrl() {
    use keylaut::core::config::BypassKey;

    let config = Config {
        bypass_key: BypassKey::Ctrl,
        ..Default::default()
    };
    let mut engine = KeylautEngine::new(config);

    // Type "ue"
    engine.process_event(KeyEvent::char_press('u', Modifiers::NONE));
    engine.process_event(KeyEvent::char_press('e', Modifiers::NONE));

    // Space with Ctrl held
    let action = engine.process_event(KeyEvent::press(
        Key::Space,
        Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
    ));
    assert_eq!(action, EngineAction::Pass);
}

#[test]
fn test_bypass_key_none_does_not_bypass() {
    use keylaut::core::config::BypassKey;

    let config = Config {
        bypass_key: BypassKey::None,
        ..Default::default()
    };
    let mut engine = KeylautEngine::new(config);

    // Type "ae"
    engine.process_event(KeyEvent::char_press('a', Modifiers::NONE));
    engine.process_event(KeyEvent::char_press('e', Modifiers::NONE));

    // Space with Alt held: since bypass_key is None, Alt does NOT bypass
    let action = engine.process_event(KeyEvent::press(
        Key::Space,
        Modifiers {
            alt: true,
            ..Modifiers::NONE
        },
    ));
    assert_eq!(
        action,
        EngineAction::Replace {
            backspaces: 2,
            text: "ä ".to_string()
        }
    );
}
