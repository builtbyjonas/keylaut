# Keylaut

### German characters. Without changing your keyboard.

**Keylaut is a tiny native utility for typing German characters on English keyboard layouts.**

Type:

```text
ae → ä
oe → ö
ue → ü
ss → ß
```

No keyboard switching.  
No autocorrect dictionary.  
No cloud.  
No Electron.  
No nonsense.  

**Just type.**

---

## Install

### Windows

```powershell
irm https://keylaut.byjonas.dev/install.ps1 | iex
```

### macOS / Linux

```bash
curl -fsSL https://keylaut.byjonas.dev/install.sh | sh
```

*Prefer building from source?*
```bash
cargo install --git https://github.com/builtbyjonas/keylaut.git
```

---

## How It Works

Keylaut monitors only the immediate keystroke stream necessary to identify intended German characters. It uses an in-memory deterministic state machine rather than aggressive dictionary replacement.

When typing naturally:

```text
Ich bin fuer dich da.  →  Ich bin für dich da.
Das ist schoen.        →  Das ist schön.
Ich gehe spaeter heim. →  Ich gehe später heim.
```

While English and international words remain untouched:

```text
aesthetic    →  aesthetic
aerospace    →  aerospace
coefficient  →  coefficient
issue        →  issue
```

---

## Lightweight by Design

Keylaut is a native Rust binary with zero bloat.

* **CPU while idle:** ~0%
* **RAM:** < 10 MB
* **Disk:** Single-digit MB binary
* **Startup:** Instant
* **Architecture:** Native OS event taps and hooks (CoreGraphics, WH_KEYBOARD_LL, evdev/X11)
* **No polling:** Sleeps until a keyboard event occurs

---

## Private by Design

Keylaut is built privacy-first by architecture.

| Property | Status |
| :--- | :--- |
| **Internet connection** | No |
| **User account** | No |
| **Telemetry / Tracking** | No |
| **Cloud processing** | No |
| **Keyboard history log** | No |

> **Your keystrokes stay on your machine.**

Keylaut never stores typed text, never records keyboard history, and never transmits data over any network.

---

## Supported Characters

| Input | Output | Example |
| :--- | :--- | :--- |
| `ae` | `ä` | `spaeter` → `später` |
| `oe` | `ö` | `schoen` → `schön` |
| `ue` | `ü` | `fuer` → `für` |
| `ss` | `ß` | `groesser` → `größer` |

### Capitalized Forms

| Input | Output | Example |
| :--- | :--- | :--- |
| `Ae` / `AE` | `Ä` | `Aerzte` → `Ärzte` |
| `Oe` / `OE` | `Ö` | `Oeffnen` → `Öffnen` |
| `Ue` / `UE` | `Ü` | `Ueber` → `Über` |

*(Note: `SS → ẞ` is intentionally not enabled by default, preserving standard orthography).*

---

## CLI Commands

```bash
# Run in foreground
keylaut

# Start as background service (configured for user login)
keylaut start

# Stop background service
keylaut stop

# Check operational and permission status
keylaut status

# Temporarily enable/disable transformation without quitting
keylaut enable
keylaut disable

# View configuration path or inspect settings
keylaut config path
keylaut config show

# Manage automatic login startup
keylaut autostart enable
keylaut autostart disable
keylaut autostart status
```

---

## Configuration

Keylaut operates out of the box with zero configuration. When needed, configuration can be customized via TOML:

* **macOS:** `~/Library/Application Support/Keylaut/config.toml`
* **Linux:** `~/.config/keylaut/config.toml`
* **Windows:** `%APPDATA%\Keylaut\config.toml`

```toml
enabled = true

[mappings]
ae = "ä"
oe = "ö"
ue = "ü"
ss = "ß"
```

---

## Documentation

* [Getting Started](docs/getting-started.md)
* [Configuration Guide](docs/configuration.md)
* [Platform Notes (Windows, macOS, Linux)](docs/platforms.md)
* [Architecture & State Machine](docs/architecture.md)
* [Troubleshooting](docs/troubleshooting.md)
* [Contributing Guide](CONTRIBUTING.md)
* [Security Policy](SECURITY.md)

---

## Built for people who type German on English keyboards.

Keylaut does one thing.

It makes:

```text
ae
oe
ue
ss
```

feel like:

```text
ä
ö
ü
ß
```

without changing your keyboard layout.

Native Rust.  
Tiny footprint.  
Completely local.  
Open source.  

### Keylaut

**Just type German.**

---

## License

MIT © [ByJonas](LICENSE)
