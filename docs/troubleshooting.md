# Troubleshooting Keylaut

---

## 1. macOS: "Keylaut could not access global keyboard input"

On macOS, background keyboard interceptors require Accessibility permissions.

### Check Service Status First:
If Keylaut is already running in the background as a service, global keyboard access is already granted and active:
```bash
keylaut status
```
When running, `keylaut status` shows:
```text
  Service:     Active / Running (PID 12345)
  Permissions: OK (Global keyboard access active in running process)
```

### If Starting in Foreground:
1. Open **System Settings** → **Privacy & Security** → **Accessibility**.
2. Locate **Keylaut** in the list.
3. If it is already there, toggle it OFF and back ON.
4. If running directly from Terminal/iTerm/IDE, ensure **Terminal** (or **iTerm2** / your IDE) has Accessibility permission enabled, or trigger the macOS permission prompt with:
   ```bash
   keylaut permissions
   ```
5. Or run Keylaut as a background service:
   ```bash
   keylaut start
   ```

---

## 2. Characters are not transforming

Check the following:
1. **Is Keylaut enabled?**
   Run `keylaut status` to verify `Enabled: Yes`. If disabled, run `keylaut enable`.
2. **Did you type a delimiter?**
   Keylaut converts characters upon reaching a word boundary (e.g. Space, Enter, Tab, or punctuation like `.`, `,`, `!`).
3. **Was the bypass key held?**
   Holding `Option` on macOS (`Alt` on Windows/Linux) while typing or pressing Space temporarily bypasses replacement. Check `Bypass key` in `keylaut status`.
4. **Is it a protected word?**
   Words that match English vocabulary (e.g., `aesthetic`, `aerospace`, `issue`, `shoe`, `blue`) are intentionally skipped to prevent corrupting non-German text.
5. **Did you use navigation or Escape?**
   Pressing Escape or Arrow keys cancels any pending candidate.

---

## 3. Autostart is not functioning

Run:
```bash
keylaut autostart status
```
If disabled, re-enable it:
```bash
keylaut autostart enable
```
* **macOS:** Verify that `~/Library/LaunchAgents/com.builtbyjonas.keylaut.plist` exists and `launchctl list | grep keylaut` shows the job.
* **Linux:** Run `systemctl --user status keylaut.service`.
* **Windows:** Check that `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\Keylaut.cmd` exists.
