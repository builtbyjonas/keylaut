# Troubleshooting Keylaut

---

## 1. macOS: "Keylaut could not access global keyboard input"

On macOS, background keyboard interceptors require Accessibility permissions.

### Solution:
1. Open **System Settings** → **Privacy & Security** → **Accessibility**.
2. Locate **Keylaut** in the list.
3. If it is already there, toggle it OFF and back ON.
4. If running directly from Terminal/iTerm, ensure **Terminal** (or **iTerm**) has Accessibility permission enabled.
5. Check status:
   ```bash
   keylaut status
   ```

---

## 2. Characters are not transforming

Check the following:
1. **Is Keylaut enabled?**
   Run `keylaut status` to verify `Enabled: Yes`. If disabled, run `keylaut enable`.
2. **Did you type a delimiter?**
   Keylaut converts characters upon reaching a word boundary (e.g. Space, Enter, Tab, or punctuation like `.`, `,`, `!`).
3. **Is it a protected word?**
   Words that match English vocabulary (e.g., `aesthetic`, `aerospace`, `issue`, `shoe`, `blue`) are intentionally skipped to prevent corrupting non-German text.
4. **Did you use navigation or Escape?**
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
