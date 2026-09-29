# Getting Started with Keylaut

Keylaut is a tiny, native background utility that allows you to type German characters (`ä`, `ö`, `ü`, `ß`) naturally while using an English keyboard layout.

---

## 1. Installation

### macOS & Linux
Run the install script in your terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/builtbyjonas/keylaut/main/scripts/install.sh | sh
```

This installs the `keylaut` binary to `~/.local/bin/keylaut` and configures automatic login startup.

### Windows
Run in PowerShell:

```powershell
irm https://raw.githubusercontent.com/builtbyjonas/keylaut/main/scripts/install.ps1 | iex
```

### From Source
```bash
cargo install --git https://github.com/builtbyjonas/keylaut.git
```

---

## 2. First Launch & Permissions

Run Keylaut in your terminal:

```bash
keylaut
```

### macOS Permissions
On macOS, global keyboard utilities require Accessibility permission:
1. When launched, macOS may prompt to grant Accessibility permissions.
2. If not prompted automatically, open:
   **System Settings → Privacy & Security → Accessibility**
3. Add or toggle on **Keylaut** (or your Terminal app if running directly).
4. Run `keylaut status` to confirm permissions are active.

---

## 3. How to Use

Simply type as you normally would. When you type:

```text
ae  →  ä
oe  →  ö
ue  →  ü
ss  →  ß
```

Keylaut will hold the candidate and transform it when you finish the word (by typing a Space, Enter, Tab, or punctuation like `.`, `,`, `!`, `?`):

* `Ich gehe spaeter nach Hause.` becomes `Ich gehe später nach Hause.`
* `Das ist schoen!` becomes `Das ist schön!`
* `Ich bin fuer dich da.` becomes `Ich bin für dich da.`

English words such as `aesthetic`, `aerospace`, `coefficient`, and `issue` are recognized by safety rules and are **not** modified.

---

## 4. Background Operation & Autostart

To keep Keylaut running in the background whenever you log into your computer:

```bash
# Enable automatic startup
keylaut autostart enable

# Check status
keylaut status

# Disable automatic startup
keylaut autostart disable
```

---

## 5. Temporarily Disabling Keylaut

If you want to suspend character transformations without quitting the process:

```bash
keylaut disable
```

To re-enable:

```bash
keylaut enable
```

---

## 6. Uninstalling

To completely remove Keylaut and its startup configuration:

### macOS / Linux
```bash
curl -fsSL https://raw.githubusercontent.com/builtbyjonas/keylaut/main/scripts/uninstall.sh | sh
```

### Windows
```powershell
irm https://raw.githubusercontent.com/builtbyjonas/keylaut/main/scripts/uninstall.ps1 | iex
```
