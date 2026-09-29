# Platform Specifics

Keylaut provides native, platform-optimized backends tailored to each operating system.

---

## 1. macOS

### Mechanism
Keylaut uses native Quartz CoreGraphics Event Taps (`CGEventTapCreate`). It installs an event tap into the session event stream to intercept keyboard events and inject Unicode replacement characters atomically.

### Required Permissions
* **Permission:** Accessibility
* **Path:** `System Settings → Privacy & Security → Accessibility`
* **Why it is needed:** macOS restricts global event observation and injection to protect user security. Keylaut uses Accessibility strictly to read keyboard events for umlaut candidates and inject replacements.

### Autostart
Configured via a user LaunchAgent:
`~/Library/LaunchAgents/com.builtbyjonas.keylaut.plist`

---

## 2. Windows

### Mechanism
Keylaut uses a native low-level keyboard hook (`SetWindowsHookExW` with `WH_KEYBOARD_LL`). Character injection is performed using `SendInput` with `KEYEVENTF_UNICODE`.

### Required Permissions
Windows does not require elevated/administrator privileges to intercept user-level keyboard input. Standard user permissions are sufficient.

### Autostart
Configured via a launcher script in the user's Startup directory:
`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\Keylaut.cmd`

---

## 3. Linux

### X11 vs Wayland
* **X11:** Global keyboard events can be observed and injected directly.
* **Wayland:** The Wayland security model intentionally prevents background processes from globally intercepting other applications' keystrokes.

### Running under Wayland
To use Keylaut on Wayland environments:
1. Run your desktop applications through an XWayland bridge, or
2. Use an input method daemon (such as IBus or Fcitx5) with Keylaut rules, or
3. Grant device access via the `input` group for evdev/uinput (requires adding your user to `/dev/uinput` permissions).

### Autostart
Configured using a systemd user service:
`~/.config/systemd/user/keylaut.service`
With automatic fallback to XDG autostart:
`~/.config/autostart/keylaut.desktop`
