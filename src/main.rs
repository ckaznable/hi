use std::path::PathBuf;

use anyhow::Result;
use argh::FromArgs;
use tokio::signal::unix::{SignalKind, signal};
use tracing::{error, info};

/// Terminal LLM chat tool
#[derive(FromArgs, Debug, PartialEq)]
struct Cli {
    /// path to config file (overrides default location)
    #[argh(option, short = 'c')]
    config: Option<PathBuf>,

    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand)]
enum Commands {
    Init(InitCommand),
    Tui(TuiCommand),
    Remote(RemoteCommand),
    Config(ConfigCommand),
}

/// Create a starter config template at the default config path
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "init")]
struct InitCommand {}

/// Start interactive terminal chat UI
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "tui")]
struct TuiCommand {}

/// Start Telegram bot long-polling mode
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "remote")]
struct RemoteCommand {}

/// Config management commands
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "config")]
struct ConfigCommand {
    #[argh(subcommand)]
    subcommand: ConfigSubcommands,
}

#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand)]
enum ConfigSubcommands {
    Validate(ValidateCommand),
}

/// Validate config by sending a test message to the configured LLM provider
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "validate")]
struct ValidateCommand {}

#[tokio::main(worker_threads = 4)]
async fn main() -> Result<()> {
    shared::logging::init_logging();
    let cli: Cli = argh::from_env();

    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::spawn(async move {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C (SIGINT), shutting down...");
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down...");
            }
        }
        std::process::exit(0);
    });

    match cli.command {
        Commands::Init(_) => {
            match shared::config::init_config() {
                Ok(path) => {
                    println!("Config template created at: {}", path.display());
                    Ok(())
                }
                Err(e) => {
                    error!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Tui(_) => {
            #[cfg(feature = "tui")]
            {
                hi_tui::run_tui(cli.config).await
            }
            #[cfg(not(feature = "tui"))]
            {
                eprintln!("TUI support is not enabled in this build.");
                eprintln!();
                eprintln!("To enable TUI, rebuild with the `tui` feature:");
                eprintln!("  cargo run -p hi-cli --features tui -- tui");
                eprintln!("  cargo install --path bin/hi --features tui");
                std::process::exit(1);
            }
        }
        Commands::Remote(_) => hi_remote::run_remote(cli.config).await,
        Commands::Config(config_cmd) => {
            match config_cmd.subcommand {
                ConfigSubcommands::Validate(_) => {
                    match hi_core::validate::validate_config(cli.config).await {
                        Ok(success) => {
                            println!("Config validation passed.");
                            println!("  Provider: {}", success.provider);
                            println!("  Model: {}", success.model);
                            println!("  Response: {}", success.response_snippet);
                            Ok(())
                        }
                        Err(err) => {
                            eprintln!("Config validation failed: {}", err.kind);
                            if let Some(ref provider) = err.provider {
                                eprintln!("  Provider: {}", provider);
                            }
                            if let Some(ref model) = err.model {
                                eprintln!("  Model: {}", model);
                            }
                            eprintln!("  Error: {}", err.message);
                            eprintln!();
                            eprintln!("  Hint: {}", err.hint);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tui_command() {
        let cli = Cli::from_args(&["hi"], &["tui"]).unwrap();
        assert_eq!(cli.command, Commands::Tui(TuiCommand {}));
        assert_eq!(cli.config, None);
    }

    #[test]
    fn test_parse_remote_command() {
        let cli = Cli::from_args(&["hi"], &["remote"]).unwrap();
        assert_eq!(cli.command, Commands::Remote(RemoteCommand {}));
        assert_eq!(cli.config, None);
    }

    #[test]
    fn test_parse_init_command() {
        let cli = Cli::from_args(&["hi"], &["init"]).unwrap();
        assert_eq!(cli.command, Commands::Init(InitCommand {}));
        assert_eq!(cli.config, None);
    }

    #[test]
    fn test_no_subcommand_fails() {
        assert!(Cli::from_args(&["hi"], &[]).is_err());
    }

    #[test]
    fn test_unknown_subcommand_fails() {
        assert!(Cli::from_args(&["hi"], &["unknown"]).is_err());
    }

    #[test]
    fn test_parse_config_validate_command() {
        let cli = Cli::from_args(&["hi"], &["config", "validate"]).unwrap();
        assert_eq!(
            cli.command,
            Commands::Config(ConfigCommand {
                subcommand: ConfigSubcommands::Validate(ValidateCommand {}),
            })
        );
    }

    #[test]
    fn test_config_without_subcommand_fails() {
        assert!(Cli::from_args(&["hi"], &["config"]).is_err());
    }

    #[test]
    fn test_parse_config_long_flag() {
        let cli = Cli::from_args(&["hi"], &["--config", "/tmp/my.json", "tui"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/tmp/my.json")));
        assert_eq!(cli.command, Commands::Tui(TuiCommand {}));
    }

    #[test]
    fn test_parse_config_short_flag() {
        let cli = Cli::from_args(&["hi"], &["-c", "/tmp/my.json", "remote"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/tmp/my.json")));
        assert_eq!(cli.command, Commands::Remote(RemoteCommand {}));
    }

    #[test]
    fn test_parse_config_flag_with_validate() {
        let cli = Cli::from_args(&["hi"], &["--config", "alt.json", "config", "validate"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("alt.json")));
        assert_eq!(
            cli.command,
            Commands::Config(ConfigCommand {
                subcommand: ConfigSubcommands::Validate(ValidateCommand {}),
            })
        );
    }
}
