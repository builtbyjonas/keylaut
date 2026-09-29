//! Main executable entry point for Keylaut.

use clap::Parser;
use keylaut::cli::{AutostartAction, Cli, Commands, ConfigAction};
use keylaut::core::config::Config;
use keylaut::core::KeylautEngine;
use keylaut::platform;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Configure minimal logging (Section 43)
    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp_secs()
        .init();

    match cli.command {
        None | Some(Commands::Run) => {
            let config = Config::load_default();
            log::info!("Starting Keylaut with enabled={}", config.enabled);

            println!("Keylaut started. German character detection active (ä ö ü ß).");
            println!("Keystrokes stay strictly local on your machine.");

            let engine = KeylautEngine::new(config);
            if let Err(err) = platform::run(engine) {
                eprintln!("\n{}", err);
                std::process::exit(1);
            }
        }

        Some(Commands::Start) => {
            platform::autostart_enable()?;
            println!("Keylaut background service started and configured for automatic startup.");
        }

        Some(Commands::Stop) => {
            platform::autostart_disable()?;
            println!("Keylaut background service stopped.");
        }

        Some(Commands::Status) => {
            println!("Keylaut Status:");
            match platform::check_permissions() {
                Ok(()) => println!("  Permissions: OK (Global keyboard access granted)"),
                Err(e) => println!("  Permissions: REQUIRED\n  {}", e.replace('\n', "\n  ")),
            }

            match platform::autostart_status() {
                Ok(true) => println!("  Autostart:   Enabled (starts automatically at login)"),
                Ok(false) => println!("  Autostart:   Disabled"),
                Err(err) => println!("  Autostart:   Unknown ({})", err),
            }

            let config = Config::load_default();
            println!(
                "  Enabled:     {}",
                if config.enabled { "Yes" } else { "No" }
            );

            if let Some(path) = Config::default_path() {
                println!("  Config path: {}", path.display());
            }
        }

        Some(Commands::Enable) => {
            let mut config = Config::load_default();
            config.enabled = true;
            let path = config.save_default()?;
            println!("Keylaut enabled in configuration ({})", path.display());
        }

        Some(Commands::Disable) => {
            let mut config = Config::load_default();
            config.enabled = false;
            let path = config.save_default()?;
            println!("Keylaut disabled in configuration ({})", path.display());
        }

        Some(Commands::Autostart(args)) => match args.action {
            AutostartAction::Enable => {
                platform::autostart_enable()?;
                println!("Keylaut autostart enabled (runs automatically at user login).");
            }
            AutostartAction::Disable => {
                platform::autostart_disable()?;
                println!("Keylaut autostart disabled.");
            }
            AutostartAction::Status => match platform::autostart_status() {
                Ok(true) => println!("Autostart is enabled."),
                Ok(false) => println!("Autostart is disabled."),
                Err(err) => eprintln!("Error checking autostart status: {}", err),
            },
        },

        Some(Commands::Config(args)) => match args.action {
            Some(ConfigAction::Path) | None => {
                if let Some(path) = Config::default_path() {
                    println!("{}", path.display());
                } else {
                    eprintln!("Could not determine platform configuration directory.");
                    std::process::exit(1);
                }
            }
            Some(ConfigAction::Show) => {
                let config = Config::load_default();
                let toml_str = toml::to_string_pretty(&config)?;
                println!("{}", toml_str);
            }
        },
    }

    Ok(())
}
