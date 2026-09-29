//! Keylaut core engine, state machine, and mapping layer.
//!
//! This module contains pure, platform-independent logic with zero OS dependencies.

pub mod config;
pub mod engine;
pub mod event;
pub mod mappings;
pub mod state;

pub use config::Config;
pub use engine::KeylautEngine;
pub use event::{Key, KeyAction, KeyEvent, Modifiers};
pub use mappings::MappingTable;
pub use state::{EngineAction, State, StateMachine};
