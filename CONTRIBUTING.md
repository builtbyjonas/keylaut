# Contributing to Keylaut

Thank you for your interest in contributing to Keylaut!

Keylaut is designed around four pillars: **Tiny. Native. Private. Instant.**  
To keep the binary small, reliable, and secure, we hold a high standard for minimalism and architectural clarity.

---

## Development Prerequisites

* Rust 1.75+ (stable toolchain)
* Standard C compiler and development headers for your platform
  * **macOS:** Xcode Command Line Tools
  * **Linux:** `build-essential`, `libx11-dev` (if working on X11)
  * **Windows:** Visual Studio C++ Build Tools

---

## Development Workflow

1. Clone the repository:
   ```bash
   git clone https://github.com/builtbyjonas/keylaut.git
   cd keylaut
   ```
2. Build the project:
   ```bash
   cargo build
   ```
3. Run formatting, linting, and tests:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-targets --all-features
   ```

Before submitting a Pull Request, all three checks must pass cleanly.

---

## Architectural Guidelines

1. **Decoupled Core:**
   The core state machine (`src/core/`) must remain pure, deterministic, and 100% platform-independent. It must never directly invoke OS-specific APIs.
2. **Platform Modules:**
   Platform-specific keyboard hooking, event injection, and autostart live exclusively in `src/platform/`.
3. **Dependency Policy:**
   Every new dependency must answer:
   * Does it solve a difficult platform problem?
   * Does it significantly reduce unsafe code?
   * Does it materially improve maintainability?
   * Does it provide a standard OS abstraction?
   Avoid GUI frameworks, web runtimes, network crates, analytics, or complex macro libraries.
4. **Privacy First:**
   Never add network communication, telemetry, disk logging of keystrokes, or full-buffer keylogging.

---

## Commit Guidelines

We follow concise conventional commit messages:

* `feat: add macOS keyboard backend`
* `fix: handle backspace after candidate`
* `refactor: isolate mapping engine`
* `docs: document Wayland limitations`
* `ci: add release matrix`
