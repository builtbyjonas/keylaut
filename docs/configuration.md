# Configuration Guide

Keylaut is zero-config by default. You can inspect or modify its settings at any time.

---

## Configuration File Location

The configuration file is formatted in standard TOML. Keylaut stores its configuration in user-local directories:

* **macOS:** `~/Library/Application Support/Keylaut/config.toml`
* **Linux:** `~/.config/keylaut/config.toml`
* **Windows:** `%APPDATA%\Keylaut\config.toml`

To locate the configuration path on your machine:

```bash
keylaut config path
```

To display active settings:

```bash
keylaut config show
```

---

## Default Configuration

```toml
# Enable or disable Keylaut globally
enabled = true

# Key sequences mapped to replacement characters
[mappings]
ae = "ä"
oe = "ö"
ue = "ü"
ss = "ß"
```

---

## Custom Mappings

You can add custom mappings by modifying the `[mappings]` section in `config.toml`.

For example, to also map uppercase sharp S or custom diacritics:

```toml
[mappings]
ae = "ä"
oe = "ö"
ue = "ü"
ss = "ß"
"SS" = "ẞ"
```

Capitalized variants (e.g. `Ae → Ä`, `AE → Ä`) are automatically generated for two-character lowercase umlaut mappings unless explicitly specified.

---

## Optional Timeout

By default, Keylaut does **not** rely on timeouts to replace text; replacement is 100% event-driven by word boundaries.

If you prefer candidates to convert after an idle delay, you can set `timeout_ms`:

```toml
# Milliseconds to wait before committing candidate
timeout_ms = 750
```
