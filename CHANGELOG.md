# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-29

### Added
- Core deterministic state machine for German umlaut conversions:
  - `ae → ä`, `oe → ö`, `ue → ü`, `ss → ß`
  - Uppercase and TitleCase variants (`Ae → Ä`, `AE → Ä`, etc.)
- False-positive safety suppression for English words (`aesthetic`, `aerospace`, `issue`, `coefficient`, etc.)
- Word boundary delimiter confirmation (`.`, `,`, `!`, `?`, `:`, `;`, `Space`, `Enter`, `Tab`)
- Graceful editing and cancellation handling (Backspace rollback, Escape cancellation, Arrow/navigation keys)
- Platform backends:
  - macOS: CoreGraphics Event Tap with LaunchAgent autostart support
  - Windows: Low-level keyboard hook (`WH_KEYBOARD_LL`) with Startup autostart support
  - Linux: Session detection (Wayland / X11) with systemd user service support
- CLI interface: `run`, `start`, `stop`, `status`, `enable`, `disable`, `autostart`, and `config`
- TOML configuration loader and saver
- POSIX and PowerShell automated install and uninstall scripts
- Comprehensive integration, edge case, and fuzzing test suites
