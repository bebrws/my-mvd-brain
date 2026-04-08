use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub command: SessionCommand,
}

#[derive(Subcommand)]
pub enum SessionCommand {
    Start { file: PathBuf, #[arg(long)] name: Option<String> },
    End { file: PathBuf },
    List { file: PathBuf, #[arg(long)] json: bool },
    View { file: PathBuf, #[arg(long = "session-id")] session_id: String },
    Checkpoint { file: PathBuf, #[arg(long)] label: Option<String> },
    Delete { file: PathBuf, #[arg(long = "session-id")] session_id: String },
    Replay { file: PathBuf, #[arg(long = "session-id")] session_id: String },
    Compare { file: PathBuf, #[arg(long)] session_a: String, #[arg(long)] session_b: String },
}

pub fn run(args: SessionArgs) -> Result<()> {
    match args.command {
        SessionCommand::Start { file, name } => {
            println!("Starting session{} for {}",
                name.as_deref().map_or(String::new(), |n| format!(" '{n}'")), file.display());
        }
        SessionCommand::End { file } => println!("Ending session for {}", file.display()),
        SessionCommand::List { file, json: _ } => println!("Sessions in {}", file.display()),
        SessionCommand::View { file, session_id } => println!("Session {session_id} in {}", file.display()),
        SessionCommand::Checkpoint { file, label } => {
            println!("Checkpoint{} in {}",
                label.as_deref().map_or(String::new(), |l| format!(" '{l}'")), file.display());
        }
        SessionCommand::Delete { file, session_id } => println!("Deleting session {session_id} from {}", file.display()),
        SessionCommand::Replay { file, session_id } => println!("Replaying session {session_id} from {}", file.display()),
        SessionCommand::Compare { file, session_a, session_b } => println!("Comparing {session_a} and {session_b} in {}", file.display()),
    }
    Ok(())
}
