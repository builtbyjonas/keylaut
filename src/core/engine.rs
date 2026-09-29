//! Keylaut core engine coordinating mappings, state machine, and configuration.

use super::config::Config;
use super::event::KeyEvent;
use super::mappings::MappingTable;
use super::state::{EngineAction, State, StateMachine};

/// The primary deterministic Keylaut engine.
pub struct KeylautEngine {
    config: Config,
    mappings: MappingTable,
    state_machine: StateMachine,
}

impl KeylautEngine {
    /// Creates a new Keylaut engine with the provided configuration.
    pub fn new(config: Config) -> Self {
        let mappings = MappingTable::from_custom(&config.mappings);
        let state_machine = StateMachine::new(config.timeout_ms, config.bypass_key);

        Self {
            config,
            mappings,
            state_machine,
        }
    }

    /// Process a normalized keyboard event.
    ///
    /// If Keylaut is disabled in configuration, all events pass through immediately.
    pub fn process_event(&mut self, event: KeyEvent) -> EngineAction {
        if !self.config.enabled {
            return EngineAction::Pass;
        }

        self.state_machine.process_event(&event, &self.mappings)
    }

    /// Returns whether Keylaut is currently enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Dynamically enables or disables the engine.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
        if !enabled {
            self.state_machine.reset();
        }
    }

    /// Reloads the engine with an updated configuration.
    pub fn reload_config(&mut self, config: Config) {
        self.mappings = MappingTable::from_custom(&config.mappings);
        self.state_machine = StateMachine::new(config.timeout_ms, config.bypass_key);
        self.config = config;
    }

    /// Returns the current state of the internal state machine.
    #[inline]
    pub fn state(&self) -> &State {
        self.state_machine.state()
    }

    /// Returns the currently accumulated word in the state machine buffer.
    #[inline]
    pub fn current_word(&self) -> &str {
        self.state_machine.current_word()
    }

    /// Resets the engine state.
    #[inline]
    pub fn reset(&mut self) {
        self.state_machine.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{Key, Modifiers};

    #[test]
    fn test_engine_disabled() {
        let config = Config {
            enabled: false,
            ..Default::default()
        };
        let mut engine = KeylautEngine::new(config);

        let action = engine.process_event(KeyEvent::char_press('a', Modifiers::NONE));
        assert_eq!(action, EngineAction::Pass);

        let action = engine.process_event(KeyEvent::char_press('e', Modifiers::NONE));
        assert_eq!(action, EngineAction::Pass);

        let action = engine.process_event(KeyEvent::press(Key::Space, Modifiers::NONE));
        assert_eq!(action, EngineAction::Pass);
    }

    #[test]
    fn test_engine_workflow() {
        let config = Config::default();
        let mut engine = KeylautEngine::new(config);

        engine.process_event(KeyEvent::char_press('a', Modifiers::NONE));
        engine.process_event(KeyEvent::char_press('e', Modifiers::NONE));
        let action = engine.process_event(KeyEvent::press(Key::Space, Modifiers::NONE));

        assert_eq!(
            action,
            EngineAction::Replace {
                backspaces: 2,
                text: "ä ".to_string()
            }
        );
    }
}
