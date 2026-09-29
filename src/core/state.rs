//! Deterministic state machine and candidate recognition for Keylaut.

use std::collections::HashSet;
use std::time::Instant;

use super::event::{Key, KeyAction, KeyEvent};
use super::mappings::MappingTable;

/// Actions emitted by the Keylaut engine in response to keyboard events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineAction {
    /// Allow the native event to pass through unmodified.
    Pass,
    /// Suppress the native event (do not deliver to active application).
    Suppress,
    /// Replace characters: send `backspaces` Backspace key events, then inject `text`.
    Replace { backspaces: usize, text: String },
}

impl EngineAction {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }

    pub fn is_suppress(&self) -> bool {
        matches!(self, Self::Suppress)
    }

    pub fn is_replace(&self) -> bool {
        matches!(self, Self::Replace { .. })
    }
}

/// The state of the Keylaut engine state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Waiting for input.
    Idle,
    /// In the middle of an input sequence with a candidate umlaut pair.
    Candidate { first: char, second: char },
    /// In the middle of typing a regular word without an active candidate.
    Typing,
}

/// Core state machine tracking candidate input sequences and delimiters.
pub struct StateMachine {
    state: State,
    word_buffer: String,
    last_event_time: Option<Instant>,
    timeout_ms: Option<u64>,
    english_false_positives: HashSet<&'static str>,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new(None)
    }
}

impl StateMachine {
    /// Maximum buffer size for word tokens to guarantee tiny memory footprint.
    pub const MAX_BUFFER_LEN: usize = 48;

    pub fn new(timeout_ms: Option<u64>) -> Self {
        Self {
            state: State::Idle,
            word_buffer: String::with_capacity(Self::MAX_BUFFER_LEN),
            last_event_time: None,
            timeout_ms,
            english_false_positives: Self::init_false_positives(),
        }
    }

    /// Returns the current state machine state.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Returns the current in-flight word buffer.
    pub fn current_word(&self) -> &str {
        &self.word_buffer
    }

    /// Returns the configured timeout in milliseconds, if any.
    pub fn timeout_ms(&self) -> Option<u64> {
        self.timeout_ms
    }

    /// Returns the timestamp of the last processed event.
    pub fn last_event_time(&self) -> Option<Instant> {
        self.last_event_time
    }

    /// Clears any in-flight buffer and resets to Idle.
    pub fn reset(&mut self) {
        self.state = State::Idle;
        self.word_buffer.clear();
        self.last_event_time = None;
    }

    /// Evaluates whether a word sequence should be rejected to prevent false positives.
    ///
    /// The fundamental safety mechanism of Keylaut:
    /// Words like `aesthetic`, `aerospace`, `coefficient`, `issue`, or sequences
    /// starting with `aes`, `aer`, etc., must not be modified.
    pub fn is_rejected(&self, word: &str) -> bool {
        let lower = word.to_lowercase();

        // 1. Check exact false positives from standard vocabulary
        if self.english_false_positives.contains(lower.as_str()) {
            return true;
        }

        // 2. Structural safety rules:
        // Prefixes that indicate non-German words:
        if lower.starts_with("aes") || lower.starts_with("aer") {
            return true;
        }
        if lower.starts_with("coef") {
            return true;
        }
        if lower.starts_with("issu") || lower.starts_with("tissu") {
            return true;
        }

        // Words starting with 'que' or 'gue' where 'ue' is not an umlaut
        if lower.starts_with("que") || lower.starts_with("gue") {
            return true;
        }

        false
    }

    /// Processes a normalized KeyEvent against the state machine and mapping table.
    pub fn process_event(&mut self, event: &KeyEvent, mappings: &MappingTable) -> EngineAction {
        // Only process key press events; key releases pass through
        if event.action != KeyAction::Press {
            return EngineAction::Pass;
        }

        // Modifiers with Ctrl or Meta (Cmd) represent shortcuts (e.g. Ctrl+C, Cmd+A)
        // Reset state and pass through without modification.
        if event.modifiers.has_command_modifier() {
            self.reset();
            return EngineAction::Pass;
        }

        self.last_event_time = Some(Instant::now());

        match &event.key {
            // Backspace rolls back one character
            Key::Backspace => {
                if !self.word_buffer.is_empty() {
                    self.word_buffer.pop();
                    self.update_candidate_state(mappings);
                } else {
                    self.state = State::Idle;
                }
                EngineAction::Pass
            }

            // Navigation and cancellation keys reset state
            Key::Escape
            | Key::Left
            | Key::Right
            | Key::Up
            | Key::Down
            | Key::Home
            | Key::End
            | Key::PageUp
            | Key::PageDown
            | Key::Delete => {
                self.reset();
                EngineAction::Pass
            }

            // Modifier keys alone don't affect buffer or trigger transformations
            Key::Modifier | Key::Other => EngineAction::Pass,

            // Space is a primary word boundary delimiter
            Key::Space => self.handle_delimiter(' ', mappings),

            // Enter / Return is a word boundary delimiter
            Key::Enter => self.handle_delimiter('\n', mappings),

            // Tab is a word boundary delimiter
            Key::Tab => self.handle_delimiter('\t', mappings),

            // Character input
            Key::Char(c) => {
                let ch = *c;
                if is_delimiter(ch) {
                    self.handle_delimiter(ch, mappings)
                } else {
                    self.handle_char(ch, mappings);
                    EngineAction::Pass
                }
            }
        }
    }

    /// Handles character additions into the word buffer.
    fn handle_char(&mut self, c: char, mappings: &MappingTable) {
        if self.word_buffer.len() >= Self::MAX_BUFFER_LEN {
            // Buffer overflow prevention: keep only the latest segment
            self.word_buffer.clear();
        }

        self.word_buffer.push(c);
        self.update_candidate_state(mappings);
    }

    /// Updates internal Candidate state based on the current buffer contents.
    fn update_candidate_state(&mut self, mappings: &MappingTable) {
        let chars: Vec<char> = self.word_buffer.chars().collect();
        if chars.len() >= 2 {
            let last_two = format!("{}{}", chars[chars.len() - 2], chars[chars.len() - 1]);
            if mappings.get(&last_two).is_some() {
                self.state = State::Candidate {
                    first: chars[chars.len() - 2],
                    second: chars[chars.len() - 1],
                };
                return;
            }
        }

        if self.word_buffer.is_empty() {
            self.state = State::Idle;
        } else {
            self.state = State::Typing;
        }
    }

    /// Handles word boundary delimiters (space, punctuation, enter, tab).
    fn handle_delimiter(&mut self, delimiter: char, mappings: &MappingTable) -> EngineAction {
        if self.word_buffer.is_empty() {
            self.reset();
            return EngineAction::Pass;
        }

        let word = std::mem::take(&mut self.word_buffer);
        self.state = State::Idle;

        // Check rejection rules before considering transformation
        if self.is_rejected(&word) {
            return EngineAction::Pass;
        }

        // Attempt transformation with current mappings
        if let Some(transformed) = mappings.transform_text(&word) {
            let word_char_count = word.chars().count();
            let replacement_text = format!("{}{}", transformed, delimiter);

            EngineAction::Replace {
                backspaces: word_char_count,
                text: replacement_text,
            }
        } else {
            EngineAction::Pass
        }
    }

    /// Initialize known English false positive words to prevent incorrect transformations.
    fn init_false_positives() -> HashSet<&'static str> {
        let mut set = HashSet::new();

        // Exact false positives listed in spec
        set.insert("aesthetic");
        set.insert("aesthetics");
        set.insert("aesthetician");
        set.insert("aesthete");
        set.insert("aerospace");
        set.insert("aerodynamic");
        set.insert("aerodynamics");
        set.insert("coefficient");
        set.insert("coefficients");
        set.insert("issue");
        set.insert("issues");
        set.insert("issued");
        set.insert("issuing");

        // Common English words containing 'ae', 'oe', 'ue', 'ss'
        set.insert("caesar");
        set.insert("archaeology");
        set.insert("encyclopaedia");
        set.insert("algae");
        set.insert("larvae");

        set.insert("shoe");
        set.insert("shoes");
        set.insert("shoemaker");
        set.insert("toe");
        set.insert("toes");
        set.insert("canoe");
        set.insert("canoes");
        set.insert("foe");
        set.insert("foes");
        set.insert("woe");
        set.insert("woes");
        set.insert("aloe");
        set.insert("aloes");
        set.insert("phoenix");
        set.insert("oboe");

        set.insert("blue");
        set.insert("clue");
        set.insert("clues");
        set.insert("true");
        set.insert("due");
        set.insert("glue");
        set.insert("sue");
        set.insert("sues");
        set.insert("sued");
        set.insert("cue");
        set.insert("cues");
        set.insert("hue");
        set.insert("hues");
        set.insert("venue");
        set.insert("venues");
        set.insert("revenue");
        set.insert("revenues");
        set.insert("value");
        set.insert("values");
        set.insert("statue");
        set.insert("statues");
        set.insert("argue");
        set.insert("argues");
        set.insert("argued");
        set.insert("rescue");
        set.insert("rescues");
        set.insert("rescued");
        set.insert("continue");
        set.insert("continues");
        set.insert("continued");
        set.insert("pursue");
        set.insert("pursues");
        set.insert("pursued");
        set.insert("league");
        set.insert("leagues");
        set.insert("tongue");
        set.insert("tongues");
        set.insert("vague");
        set.insert("plague");
        set.insert("dialogue");
        set.insert("catalogue");
        set.insert("fatigue");
        set.insert("mosque");
        set.insert("opaque");
        set.insert("unique");
        set.insert("technique");
        set.insert("techniques");
        set.insert("cheque");
        set.insert("queue");
        set.insert("queues");

        set.insert("fuel");
        set.insert("fuels");
        set.insert("cruel");
        set.insert("duel");
        set.insert("duels");
        set.insert("fluent");
        set.insert("affluent");
        set.insert("influence");

        set.insert("pass");
        set.insert("class");
        set.insert("grass");
        set.insert("glass");
        set.insert("less");
        set.insert("miss");
        set.insert("kiss");
        set.insert("boss");
        set.insert("cross");
        set.insert("loss");
        set.insert("process");
        set.insert("access");
        set.insert("success");
        set.insert("business");
        set.insert("address");
        set.insert("possible");
        set.insert("impossible");
        set.insert("session");
        set.insert("lesson");
        set.insert("assist");
        set.insert("asset");
        set.insert("assets");
        set.insert("assume");
        set.insert("assert");
        set.insert("message");
        set.insert("pressure");
        set.insert("essential");
        set.insert("passion");
        set.insert("mission");
        set.insert("discussion");
        set.insert("profession");
        set.insert("confess");
        set.insert("express");
        set.insert("impress");
        set.insert("depress");
        set.insert("oppress");
        set.insert("suppress");
        set.insert("progress");
        set.insert("congress");
        set.insert("witness");
        set.insert("fitness");
        set.insert("illness");
        set.insert("darkness");
        set.insert("goodness");
        set.insert("sadness");
        set.insert("happiness");
        set.insert("weakness");
        set.insert("stress");
        set.insert("chess");
        set.insert("dress");
        set.insert("bless");

        set
    }
}

/// Returns true if the character is considered a punctuation or word boundary delimiter.
pub fn is_delimiter(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t'
            | '\n'
            | '\r'
            | '.'
            | ','
            | '!'
            | '?'
            | ':'
            | ';'
            | '-'
            | '_'
            | '"'
            | '\''
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '/'
            | '\\'
            | '|'
            | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '*'
            | '+'
            | '='
            | '<'
            | '>'
            | '~'
            | '`'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::Modifiers;

    fn type_string(
        machine: &mut StateMachine,
        mappings: &MappingTable,
        text: &str,
    ) -> Vec<EngineAction> {
        let mut actions = Vec::new();
        for c in text.chars() {
            let key = match c {
                ' ' => Key::Space,
                '\n' => Key::Enter,
                '\t' => Key::Tab,
                other => Key::Char(other),
            };
            let event = KeyEvent::press(key, Modifiers::NONE);
            actions.push(machine.process_event(&event, mappings));
        }
        actions
    }

    #[test]
    fn test_ae_space_replacement() {
        let mut machine = StateMachine::default();
        let mappings = MappingTable::with_defaults();

        let actions = type_string(&mut machine, &mappings, "ae ");
        assert_eq!(
            actions.last(),
            Some(&EngineAction::Replace {
                backspaces: 2,
                text: "ä ".to_string()
            })
        );
    }

    #[test]
    fn test_schoen_exclamation() {
        let mut machine = StateMachine::default();
        let mappings = MappingTable::with_defaults();

        let actions = type_string(&mut machine, &mappings, "schoen!");
        assert_eq!(
            actions.last(),
            Some(&EngineAction::Replace {
                backspaces: 6,
                text: "schön!".to_string()
            })
        );
    }

    #[test]
    fn test_fuer_period() {
        let mut machine = StateMachine::default();
        let mappings = MappingTable::with_defaults();

        let actions = type_string(&mut machine, &mappings, "fuer.");
        assert_eq!(
            actions.last(),
            Some(&EngineAction::Replace {
                backspaces: 4,
                text: "für.".to_string()
            })
        );
    }

    #[test]
    fn test_groesser_comma() {
        let mut machine = StateMachine::default();
        let mappings = MappingTable::with_defaults();

        let actions = type_string(&mut machine, &mappings, "groesser,");
        assert_eq!(
            actions.last(),
            Some(&EngineAction::Replace {
                backspaces: 8,
                text: "größer,".to_string()
            })
        );
    }

    #[test]
    fn test_false_positive_rejection() {
        let mut machine = StateMachine::default();
        let mappings = MappingTable::with_defaults();

        // aesthetic must remain untouched
        let actions = type_string(&mut machine, &mappings, "aesthetic ");
        assert_eq!(actions.last(), Some(&EngineAction::Pass));

        // aerospace must remain untouched
        let actions = type_string(&mut machine, &mappings, "aerospace ");
        assert_eq!(actions.last(), Some(&EngineAction::Pass));

        // aerodynamic must remain untouched
        let actions = type_string(&mut machine, &mappings, "aerodynamic ");
        assert_eq!(actions.last(), Some(&EngineAction::Pass));

        // coefficient must remain untouched
        let actions = type_string(&mut machine, &mappings, "coefficient ");
        assert_eq!(actions.last(), Some(&EngineAction::Pass));

        // issue must remain untouched
        let actions = type_string(&mut machine, &mappings, "issue ");
        assert_eq!(actions.last(), Some(&EngineAction::Pass));
    }

    #[test]
    fn test_aes_and_aer_suppression() {
        let mut machine = StateMachine::default();
        let mappings = MappingTable::with_defaults();

        let actions = type_string(&mut machine, &mappings, "aes ");
        assert_eq!(actions.last(), Some(&EngineAction::Pass));

        let actions = type_string(&mut machine, &mappings, "aer ");
        assert_eq!(actions.last(), Some(&EngineAction::Pass));
    }

    #[test]
    fn test_backspace_editing() {
        let mut machine = StateMachine::default();
        let mappings = MappingTable::with_defaults();

        type_string(&mut machine, &mappings, "fuer");
        assert_eq!(machine.current_word(), "fuer");

        machine.process_event(&KeyEvent::press(Key::Backspace, Modifiers::NONE), &mappings);
        assert_eq!(machine.current_word(), "fue");

        type_string(&mut machine, &mappings, "r.");
        assert_eq!(machine.state(), &State::Idle);
    }

    #[test]
    fn test_escape_cancels_candidate() {
        let mut machine = StateMachine::default();
        let mappings = MappingTable::with_defaults();

        type_string(&mut machine, &mappings, "ae");
        assert!(matches!(machine.state(), State::Candidate { .. }));

        machine.process_event(&KeyEvent::press(Key::Escape, Modifiers::NONE), &mappings);
        assert_eq!(machine.state(), &State::Idle);
        assert_eq!(machine.current_word(), "");
    }
}
