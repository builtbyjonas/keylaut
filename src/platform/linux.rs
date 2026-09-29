//! Linux platform backend with environment detection (X11 / Wayland) and user-level startup.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::core::KeylautEngine;

/// Detected Linux display environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxEnvironment {
    X11,
    Wayland,
    Headless,
}

impl LinuxEnvironment {
    /// Detects whether the current session is X11, Wayland, or Headless.
    pub fn detect() -> Self {
        if let Ok(val) = std::env::var("WAYLAND_DISPLAY") {
            if !val.is_empty() {
                return Self::Wayland;
            }
        }

        if let Ok(val) = std::env::var("XDG_SESSION_TYPE") {
            if val.eq_ignore_ascii_case("wayland") {
                return Self::Wayland;
            } else if val.eq_ignore_ascii_case("x11") {
                return Self::X11;
            }
        }

        if std::env::var("DISPLAY").is_ok() {
            Self::X11
        } else {
            Self::Headless
        }
    }
}

pub fn check_permissions() -> Result<(), String> {
    match LinuxEnvironment::detect() {
        LinuxEnvironment::Wayland => {
            log::warn!(
                "Running under Wayland. Global keyboard interception is restricted by the Wayland security model."
            );
            Ok(())
        }
        LinuxEnvironment::X11 => Ok(()),
        LinuxEnvironment::Headless => Err(
            "No graphical display detected (neither WAYLAND_DISPLAY nor DISPLAY is set)."
                .to_string(),
        ),
    }
}

pub fn run(_engine: KeylautEngine) -> Result<(), String> {
    let env = LinuxEnvironment::detect();
    match env {
        LinuxEnvironment::Wayland => {
            Err(
                "Keylaut global interception under Wayland requires an input daemon or XWayland bridge.\n\n\
                Wayland intentionally isolates window input for security.\n\
                Please refer to docs/platforms.md#linux-wayland for configuration guidance."
                    .to_string(),
            )
        }
        LinuxEnvironment::X11 => {
            log::info!("Starting Keylaut on Linux (X11).");
            // In X11 environments without active event loop thread, wait for interrupt
            println!("Keylaut Linux X11 backend initialized.");
            Ok(())
        }
        LinuxEnvironment::Headless => {
            Err("Cannot run Keylaut without an active graphical session.".to_string())
        }
    }
}

pub fn stop() {
    // Linux stop signal
}

// ---------------------------------------------------------------------------
// Linux Autostart via systemd --user or XDG autostart
// ---------------------------------------------------------------------------

fn systemd_service_path() -> Result<PathBuf, std::io::Error> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine config directory",
        )
    })?;
    Ok(config_dir
        .join("systemd")
        .join("user")
        .join("keylaut.service"))
}

fn xdg_autostart_path() -> Result<PathBuf, std::io::Error> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine config directory",
        )
    })?;
    Ok(config_dir.join("autostart").join("keylaut.desktop"))
}

pub fn autostart_enable() -> Result<(), std::io::Error> {
    let current_exe = std::env::current_exe()?;
    let exe_str = current_exe.to_string_lossy();

    // Prefer systemd --user service if systemd is available
    if Command::new("systemctl").arg("--version").output().is_ok() {
        let service_path = systemd_service_path()?;
        if let Some(parent) = service_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let service_content = format!(
            "[Unit]\n\
            Description=Keylaut German Character Keyboard Utility\n\
            Documentation=https://github.com/builtbyjonas/keylaut\n\
            After=graphical-session.target\n\n\
            [Service]\n\
            ExecStart={} run\n\
            Restart=on-failure\n\
            RestartSec=3\n\n\
            [Install]\n\
            WantedBy=default.target\n",
            exe_str
        );

        fs::write(&service_path, service_content)?;

        let _ = Command::new("systemctl")
            .arg("--user")
            .arg("daemon-reload")
            .status();

        let status = Command::new("systemctl")
            .arg("--user")
            .arg("enable")
            .arg("--now")
            .arg("keylaut.service")
            .status()?;

        if status.success() {
            return Ok(());
        }
    }

    // Fallback: XDG Desktop autostart
    let desktop_path = xdg_autostart_path()?;
    if let Some(parent) = desktop_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let desktop_content = format!(
        "[Desktop Entry]\n\
        Type=Application\n\
        Name=Keylaut\n\
        Comment=German characters without changing your keyboard layout\n\
        Exec={} run\n\
        Terminal=false\n\
        X-GNOME-Autostart-enabled=true\n",
        exe_str
    );

    fs::write(&desktop_path, desktop_content)?;
    Ok(())
}

pub fn autostart_disable() -> Result<(), std::io::Error> {
    if Command::new("systemctl").arg("--version").output().is_ok() {
        let _ = Command::new("systemctl")
            .arg("--user")
            .arg("disable")
            .arg("--now")
            .arg("keylaut.service")
            .status();

        if let Ok(service_path) = systemd_service_path() {
            if service_path.exists() {
                let _ = fs::remove_file(&service_path);
            }
        }
    }

    if let Ok(desktop_path) = xdg_autostart_path() {
        if desktop_path.exists() {
            let _ = fs::remove_file(&desktop_path);
        }
    }

    Ok(())
}

pub fn autostart_status() -> Result<bool, std::io::Error> {
    if let Ok(service_path) = systemd_service_path() {
        if service_path.exists() {
            return Ok(true);
        }
    }
    if let Ok(desktop_path) = xdg_autostart_path() {
        if desktop_path.exists() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Checks if a background Keylaut service or daemon is running on Linux.
pub fn running_pid() -> Option<u32> {
    let output = Command::new("systemctl")
        .args(["--user", "show", "keylaut.service", "--property=MainPID"])
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        if let Some(pid_str) = text.trim().strip_prefix("MainPID=") {
            if let Ok(pid) = pid_str.parse::<u32>() {
                if pid > 0 && pid != std::process::id() {
                    return Some(pid);
                }
            }
        }
    }
    None
}

/// Stops a running Keylaut process on Linux.
pub fn stop_pid(pid: u32) {
    unsafe {
        let _ = libc::kill(pid as i32, libc::SIGTERM);
    }
}
