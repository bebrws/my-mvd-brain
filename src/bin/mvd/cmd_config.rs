use anyhow::{Context, Result};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Set a configuration value
    Set { key: String, value: String },
    /// Get a configuration value
    Get { key: String },
    /// List all configuration values
    List,
    /// Remove a configuration value
    Unset { key: String },
    /// Check the configuration is valid
    Check,
}

pub fn run(args: ConfigArgs) -> Result<()> {
    let config_path = crate::common::config_file_path()?;
    match args.command {
        ConfigCommand::Set { key, value } => {
            let mut config = load_config(&config_path)?;
            config.insert(key.clone(), serde_json::Value::String(value.clone()));
            save_config(&config_path, &config)?;
            println!("Set {key} = {value}");
        }
        ConfigCommand::Get { key } => {
            let config = load_config(&config_path)?;
            if let Some(val) = config.get(&key) {
                println!("{val}");
            } else {
                eprintln!("Key not found: {key}");
                std::process::exit(1);
            }
        }
        ConfigCommand::List => {
            let config = load_config(&config_path)?;
            if config.is_empty() {
                println!("No configuration set. Config file: {}", config_path.display());
            } else {
                for (k, v) in &config {
                    println!("{k} = {v}");
                }
            }
        }
        ConfigCommand::Unset { key } => {
            let mut config = load_config(&config_path)?;
            if config.remove(&key).is_some() {
                save_config(&config_path, &config)?;
                println!("Removed {key}");
            } else {
                eprintln!("Key not found: {key}");
            }
        }
        ConfigCommand::Check => {
            let config = load_config(&config_path)?;
            println!("Config file: {}", config_path.display());
            println!("Keys: {}", config.len());
            println!("Config is valid.");
        }
    }
    Ok(())
}

fn load_config(path: &std::path::Path) -> Result<serde_json::Map<String, serde_json::Value>> {
    if !path.exists() { return Ok(serde_json::Map::new()); }
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    let val: serde_json::Value = serde_json::from_str(&data).context("Invalid config JSON")?;
    val.as_object().cloned().ok_or_else(|| anyhow::anyhow!("Config is not a JSON object"))
}

fn save_config(path: &std::path::Path, config: &serde_json::Map<String, serde_json::Value>) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(&serde_json::Value::Object(config.clone()))?;
    std::fs::write(path, data)?;
    Ok(())
}
