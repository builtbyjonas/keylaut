# Security Policy

## Security & Privacy Guarantees

Because Keylaut operates as a background utility intercepting keyboard input to perform character conversions, security and privacy are treated as first-class constraints.

### Architectural Invariants

* **Zero Network Communication:** Keylaut does not include any networking code, HTTP clients, socket connections, or remote communication libraries.
* **No Telemetry / Analytics:** Zero data is gathered, transmitted, or recorded about your system, typing habits, or application usage.
* **No Keystroke Logging:** Keylaut retains only a transient in-memory buffer of the current word in progress (cleared upon word boundary, navigation, or cancellation). It never writes keystrokes to disk.
* **No Remote Code Execution:** Keylaut does not evaluate dynamic scripts, shell commands from config, or remote configuration files.
* **Least Privilege:** Keylaut does not require administrator/root privileges for normal operation or installation.

---

## Reporting a Vulnerability

If you discover a security vulnerability or privacy concern in Keylaut, please report it privately:

1. **Email:** Contact [me@byjonas.dev](mailto:me@byjonas.dev).
2. **Details:** Include a description of the issue, affected platforms/versions, and reproduction steps.
3. **Response:** You will receive an acknowledgment within 48 hours.

Please do not open public GitHub issues for security vulnerabilities before they have been resolved.
