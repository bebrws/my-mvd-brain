use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ModelsArgs {
    #[command(subcommand)]
    pub command: ModelsCommand,
}

#[derive(Subcommand)]
pub enum ModelsCommand {
    /// List available models
    List { #[arg(long)] json: bool },
    /// Verify model integrity
    Verify { #[arg(long)] json: bool },
}

pub fn run(args: ModelsArgs) -> Result<()> {
    match args.command {
        ModelsCommand::List { json: _ } => {
            println!("Installed models:");
            #[cfg(feature = "vec")]
            {
                println!("\nAvailable text embedding models:");
                for info in memvid_core::TEXT_EMBED_MODELS {
                    println!("  {}", info.name);
                }
            }
            Ok(())
        }
        ModelsCommand::Verify { json: _ } => {
            let models_dir = crate::common::config_dir()?.join("models");
            let options = memvid_core::ModelVerifyOptions::default();
            let reports = memvid_core::verify_models(&models_dir, &options)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            if reports.is_empty() {
                println!("No models found in {}", models_dir.display());
            } else {
                for report in &reports {
                    println!("  {} — {:?}", report.digest, report.status);
                }
            }
            Ok(())
        }
    }
}
