# Architecture & Design

Keylaut is designed around a strictly decoupled architecture separating platform hooks from core state machine transformations.

---

## High-Level Architecture

```text
               ┌────────────────────────────────────────────────────────┐
               │                      Keylaut Core                      │
               │                                                        │
               │  • State Machine (Idle, Candidate, Typing)             │
               │  • Mapping Table (ae → ä, etc.)                        │
               │  • False-Positive Safety Suppression (aer, aes, etc.)  │
               │  • Delimiter & Word Boundary Detection                 │
               └───────────────────────────┬────────────────────────────┘
                                           │
                        Normalized KeyEvent │ EngineAction (Pass / Suppress / Replace)
                                           │
        ┌──────────────────────────────────┼──────────────────────────────────┐
        │                                  │                                  │
        ▼                                  ▼                                  ▼
┌───────────────┐                  ┌───────────────┐                  ┌───────────────┐
│     macOS     │                  │    Windows    │                  │     Linux     │
│ CoreGraphics  │                  │ WH_KEYBOARD_LL│                  │ X11 / evdev   │
│   Event Tap   │                  │   Hook Proc   │                  │    Backend    │
└───────────────┘                  └───────────────┘                  └───────────────┘
```

---

## Event Pipeline

```text
Physical Keystroke
       ↓
OS Keyboard Hook (macOS / Windows / Linux)
       ↓
Event Normalization (KeyEvent, Modifiers)
       ↓
Keylaut State Machine
       ↓
Safety / False-Positive Checks
       ↓
Mapping Table Lookup
       ↓
Engine Action
    ↙        ↘
 Pass         Replace { backspaces, text }
```

---

## Why a State Machine Instead of Autocorrect?

Traditional autocorrect engines:
1. Maintain dictionary databases
2. Constantly compare typed words against dictionaries
3. Make speculative guesses that frequently correct words you intended to keep

Keylaut does not use a dictionary. Instead, it uses a **deterministic state machine**:
* It waits until the sequence has completed (at a space, enter, or punctuation delimiter).
* If the word matches a safety suppression pattern (such as `aerospace` or `aesthetic`), it is passed through unchanged.
* If confirmed, Keylaut replaces the original characters with their German equivalent atomically using native OS Backspaces and Unicode injection.
* Keystrokes are never logged to disk or buffered beyond the immediate word.
