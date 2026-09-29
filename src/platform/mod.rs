//! Platform-specific keyboard interception, event injection, and autostart implementations.

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::core::KeylautEngine;

/// Checks whether global keyboard interception permissions are available.
pub fn check_permissions() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos::check_permissions()
    }

    #[cfg(target_os = "windows")]
    {
        windows::check_permissions()
    }

    #[cfg(target_os = "linux")]
    {
        linux::check_permissions()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Unsupported operating system for Keylaut global keyboard interception.".to_string())
    }
}

/// Runs the native platform keyboard loop.
pub fn run(engine: KeylautEngine) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos::run(engine)
    }

    #[cfg(target_os = "windows")]
    {
        windows::run(engine)
    }

    #[cfg(target_os = "linux")]
    {
        linux::run(engine)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Unsupported platform.".to_string())
    }
}

/// Enables system startup / autostart for the current user.
pub fn autostart_enable() -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        macos::autostart_enable()
    }

    #[cfg(target_os = "windows")]
    {
        windows::autostart_enable()
    }

    #[cfg(target_os = "linux")]
    {
        linux::autostart_enable()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Autostart is not supported on this platform.",
        ))
    }
}

/// Disables system startup / autostart for the current user.
pub fn autostart_disable() -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        macos::autostart_disable()
    }

    #[cfg(target_os = "windows")]
    {
        windows::autostart_disable()
    }

    #[cfg(target_os = "linux")]
    {
        linux::autostart_disable()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Autostart is not supported on this platform.",
        ))
    }
}

/// Returns whether autostart is currently enabled for Keylaut.
pub fn autostart_status() -> Result<bool, std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        macos::autostart_status()
    }

    #[cfg(target_os = "windows")]
    {
        windows::autostart_status()
    }

    #[cfg(target_os = "linux")]
    {
        linux::autostart_status()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(false)
    }
}

/// Returns the PID of any currently active Keylaut process or service.
pub fn running_pid() -> Option<u32> {
    #[cfg(target_os = "macos")]
    {
        macos::running_pid()
    }

    #[cfg(target_os = "windows")]
    {
        windows::running_pid()
    }

    #[cfg(target_os = "linux")]
    {
        linux::running_pid()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

/// Terminates a running Keylaut process.
pub fn stop_pid(pid: u32) {
    #[cfg(target_os = "macos")]
    {
        macos::stop_pid(pid);
    }

    #[cfg(target_os = "windows")]
    {
        windows::stop_pid(pid);
    }

    #[cfg(target_os = "linux")]
    {
        linux::stop_pid(pid);
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = pid;
    }
}

/// Requests platform keyboard/accessibility permissions.
pub fn request_permissions() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::request_permissions()
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}
