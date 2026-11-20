use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG_PATH: &str = "/etc/ark/system-manager.toml";

#[derive(Debug, Parser)]
#[command(
    name = "ark-system-manager",
    version,
    about = "Manage Lyd Ark system configuration and standardized storage"
)]
struct Cli {
    /// Configuration file (also configurable with ARK_SYSTEM_MANAGER_CONFIG).
    #[arg(long, global = true, env = "ARK_SYSTEM_MANAGER_CONFIG")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Show basic System Manager and host information.
    Info,
    /// Inspect the System Manager configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Inspect standardized storage locations.
    Storage {
        #[command(subcommand)]
        command: StorageCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Print the configuration file path.
    Path,
    /// Print the effective configuration as TOML.
    Show,
}

#[derive(Debug, Subcommand)]
enum StorageCommand {
    /// List logical storage locations and their configured paths.
    List,
    /// Show whether each configured location currently exists.
    Status,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct AppConfig {
    storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct StorageConfig {
    root: PathBuf,
    locations: BTreeMap<String, PathBuf>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        let root = PathBuf::from("/ark");
        let locations = [
            ("Containers", root.join("Containers")),
            ("Media", root.join("Media")),
            ("Share", root.join("Share")),
            ("Configs", root.join("Configs")),
        ]
        .into_iter()
        .map(|(name, path)| (name.to_string(), path))
        .collect();

        Self { root, locations }
    }
}

impl AppConfig {
    fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("Unable to read configuration at {}", path.display()))?;
        let config: Self = toml::from_str(&contents)
            .with_context(|| format!("Unable to parse configuration at {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if !self.storage.root.is_absolute() {
            bail!("storage.root must be an absolute path");
        }

        for (name, path) in &self.storage.locations {
            if !path.is_absolute() {
                bail!("storage location '{name}' must use an absolute path");
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = cli
        .config
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH));
    let config = AppConfig::load(&config_path)?;

    match cli.command {
        Command::Info => {
            println!("Lyd Ark System Manager {}", env!("CARGO_PKG_VERSION"));
            println!("Operating system: {}", std::env::consts::OS);
            println!("Architecture: {}", std::env::consts::ARCH);
            println!("Configuration: {}", config_path.display());
            println!("Storage root: {}", config.storage.root.display());
        }
        Command::Config { command } => match command {
            ConfigCommand::Path => println!("{}", config_path.display()),
            ConfigCommand::Show => {
                let output = toml::to_string_pretty(&config)
                    .context("Unable to serialize effective configuration")?;
                print!("{output}");
            }
        },
        Command::Storage { command } => match command {
            StorageCommand::List => {
                println!("{:<14} PATH", "LOCATION");
                for (name, path) in &config.storage.locations {
                    println!("{name:<14} {}", path.display());
                }
            }
            StorageCommand::Status => {
                println!("{:<14} {:<12} PATH", "LOCATION", "STATUS");
                for (name, path) in &config.storage.locations {
                    let status = match fs::symlink_metadata(path) {
                        Ok(metadata) if metadata.file_type().is_symlink() => "SYMLINK",
                        Ok(metadata) if metadata.is_dir() => "DIRECTORY",
                        Ok(_) => "OTHER",
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "MISSING",
                        Err(_) => "ERROR",
                    };
                    println!("{name:<14} {status:<12} {}", path.display());
                }
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_use_ark_storage_root() {
        let config = AppConfig::default();
        assert_eq!(config.storage.root, PathBuf::from("/ark"));
        assert_eq!(
            config.storage.locations.get("Media"),
            Some(&PathBuf::from("/ark/Media"))
        );
        assert_eq!(
            config.storage.locations.get("Share"),
            Some(&PathBuf::from("/ark/Share"))
        );
    }

    #[test]
    fn rejects_relative_storage_root() {
        let config = AppConfig {
            storage: StorageConfig {
                root: PathBuf::from("ark"),
                ..StorageConfig::default()
            },
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_relative_location() {
        let mut config = AppConfig::default();
        config
            .storage
            .locations
            .insert("Media".to_string(), PathBuf::from("Media"));
        assert!(config.validate().is_err());
    }

    #[test]
    fn loads_explicit_configuration() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("system-manager.toml");
        fs::write(
            &path,
            "[storage]\nroot = \"/ark\"\n[storage.locations]\nMedia = \"/mnt/raid/Media\"\n",
        )
        .expect("write configuration");

        let config = AppConfig::load(&path).expect("load configuration");
        assert_eq!(
            config.storage.locations.get("Media"),
            Some(&PathBuf::from("/mnt/raid/Media"))
        );
    }
}
