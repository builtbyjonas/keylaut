#!/bin/sh
# Keylaut uninstaller for macOS and Linux
# https://github.com/builtbyjonas/keylaut

set -eu

INSTALL_DIR="${HOME}/.local/bin"
BINARY_PATH="${INSTALL_DIR}/keylaut"

echo "==> Uninstalling Keylaut..."

# Disable and remove autostart if keylaut binary exists
if [ -x "$BINARY_PATH" ]; then
    "$BINARY_PATH" autostart disable >/dev/null 2>&1 || true
fi

# Remove macOS LaunchAgent plist if present
LAUNCH_AGENT="${HOME}/Library/LaunchAgents/com.builtbyjonas.keylaut.plist"
if [ -f "$LAUNCH_AGENT" ]; then
    launchctl unload "$LAUNCH_AGENT" >/dev/null 2>&1 || true
    rm -f "$LAUNCH_AGENT"
fi

# Remove Linux systemd service if present
SYSTEMD_SERVICE="${HOME}/.config/systemd/user/keylaut.service"
if [ -f "$SYSTEMD_SERVICE" ]; then
    systemctl --user disable --now keylaut.service >/dev/null 2>&1 || true
    rm -f "$SYSTEMD_SERVICE"
fi

# Remove Linux desktop autostart entry if present
XDG_DESKTOP="${HOME}/.config/autostart/keylaut.desktop"
if [ -f "$XDG_DESKTOP" ]; then
    rm -f "$XDG_DESKTOP"
fi

# Remove binary
if [ -f "$BINARY_PATH" ]; then
    rm -f "$BINARY_PATH"
    echo "==> Removed binary: $BINARY_PATH"
fi

echo "Keylaut has been successfully uninstalled."
