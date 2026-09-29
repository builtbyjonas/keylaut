//! User configuration for Keylaut.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_bypass_key() -> BypassKey {
    BypassKey::Alt
}

fn default_mappings() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("ae".to_string(), "ä".to_string());
    map.insert("oe".to_string(), "ö".to_string());
    map.insert("ue".to_string(), "ü".to_string());
    map.insert("ss".to_string(), "ß".to_string());
    map
}

/// Modifier key held while typing to temporarily bypass character replacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BypassKey {
    /// Option on macOS, Alt on Windows and Linux.
    #[serde(alias = "option")]
    #[default]
    Alt,
    /// Control key.
    #[serde(alias = "control")]
    Ctrl,
    /// Shift key.
    Shift,
    /// Command key on macOS, Windows/Super key on Windows and Linux.
    #[serde(alias = "cmd", alias = "command")]
    Meta,
    /// Bypass disabled (all words subject to normal mapping rules).
    None,
}

impl BypassKey {
    /// Checks whether this bypass modifier is currently active in the given modifier state.
    pub fn is_active(&self, modifiers: &crate::core::event::Modifiers) -> bool {
        match self {
            Self::Alt => modifiers.alt,
            Self::Ctrl => modifiers.ctrl,
            Self::Shift => modifiers.shift,
            Self::Meta => modifiers.meta,
            Self::None => false,
        }
    }

    /// User-friendly display name of the bypass key.
    pub fn display_name(&self) -> &'static str {
        match self {
            #[cfg(target_os = "macos")]
            Self::Alt => "Option (⌥)",
            #[cfg(not(target_os = "macos"))]
            Self::Alt => "Alt",
            Self::Ctrl => "Control",
            Self::Shift => "Shift",
            #[cfg(target_os = "macos")]
            Self::Meta => "Command (⌘)",
            #[cfg(not(target_os = "macos"))]
            Self::Meta => "Meta / Super",
            Self::None => "None (disabled)",
        }
    }
}

/// Keylaut application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// Whether Keylaut text transformation is globally enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Configured character mappings.
    #[serde(default = "default_mappings")]
    pub mappings: HashMap<String, String>,

    /// Optional timeout in milliseconds for candidate confirmation (None = disabled by default).
    #[serde(default)]
    pub timeout_ms: Option<u64>,

    /// Whether startup autostart is requested.
    #[serde(default)]
    pub autostart: bool,

    /// Key to hold while typing to bypass German character replacement.
    /// Default: "alt" (Option on macOS, Alt on Windows/Linux).
    /// Options: "alt", "option", "ctrl", "control", "shift", "none".
    #[serde(default = "default_bypass_key")]
    pub bypass_key: BypassKey,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            mappings: default_mappings(),
            timeout_ms: None,
            autostart: false,
            bypass_key: BypassKey::default(),
        }
    }
}

impl Config {
    /// Returns the standard platform configuration file path.
    ///
    /// - Linux: `~/.config/keylaut/config.toml`
    /// - macOS: `~/Library/Application Support/Keylaut/config.toml`
    /// - Windows: `%APPDATA%\Keylaut\config.toml`
    pub fn default_path() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            dirs::data_dir().map(|p| p.join("Keylaut").join("config.toml"))
        }

        #[cfg(target_os = "windows")]
        {
            dirs::config_dir().map(|p| p.join("Keylaut").join("config.toml"))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            dirs::config_dir().map(|p| p.join("keylaut").join("config.toml"))
        }
    }

    /// Loads the configuration from the given path, or creates and saves the default
    /// if the file does not exist yet.
    pub fn load_or_create(path: &Path) -> Result<Self, std::io::Error> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            toml::from_str(&content).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse config file at {:?}: {}", path, e),
                )
            })
        } else {
            let default_config = Self::default();
            default_config.save(path)?;
            Ok(default_config)
        }
    }

    /// Loads configuration from default platform path, or returns default config.
    pub fn load_default() -> Self {
        if let Some(path) = Self::default_path() {
            match Self::load_or_create(&path) {
                Ok(cfg) => cfg,
                Err(err) => {
                    log::warn!(
                        "Could not load config from {:?}, using defaults: {}",
                        path,
                        err
                    );
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    /// Saves the configuration to the specified path, creating directories as needed.
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::other(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, toml_string)
    }

    /// Saves the current configuration to the default platform path.
    pub fn save_default(&self) -> Result<PathBuf, std::io::Error> {
        let path = Self::default_path().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Unable to resolve default platform config directory",
            )
        })?;
        self.save(&path)?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config, deserialized);
    }
}
