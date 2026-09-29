//! Normalized keyboard event representations.

/// The action of a key event: Press or Release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Press,
    Release,
}

/// Modifier keys state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Cmd on macOS, Win on Windows, Super on Linux
}

impl Modifiers {
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        meta: false,
    };

    /// Returns true if any command/control modifier is active (ignoring shift/alt).
    #[inline]
    pub fn has_command_modifier(&self) -> bool {
        self.ctrl || self.meta
    }
}

/// Normalized key representations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Backspace,
    Delete,
    Enter,
    Tab,
    Space,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Modifier,
    Other,
}

/// A normalized keyboard event consumed by the Keylaut core engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
    pub action: KeyAction,
}

impl KeyEvent {
    /// Creates a key press event for a character.
    pub fn char_press(c: char, modifiers: Modifiers) -> Self {
        Self {
            key: Key::Char(c),
            modifiers,
            action: KeyAction::Press,
        }
    }

    /// Creates a simple key press event.
    pub fn press(key: Key, modifiers: Modifiers) -> Self {
        Self {
            key,
            modifiers,
            action: KeyAction::Press,
        }
    }

    /// Creates a key release event.
    pub fn release(key: Key, modifiers: Modifiers) -> Self {
        Self {
            key,
            modifiers,
            action: KeyAction::Release,
        }
    }
}
