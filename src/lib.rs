//! Keylaut: A tiny, native keyboard utility that makes German characters effortless
//! on English keyboard layouts.

pub mod cli;
pub mod core;
pub mod platform;

pub use core::{Config, EngineAction, KeyEvent, KeylautEngine};
