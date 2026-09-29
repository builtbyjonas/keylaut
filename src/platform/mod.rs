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
