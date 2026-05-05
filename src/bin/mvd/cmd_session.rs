use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use crate::common::WriteOpts;

#[derive(Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub command: SessionCommand,
}

#[derive(Subcommand)]
pub enum SessionCommand {
    /// Start a new recording session
    Start {
        file: PathBuf,
        #[arg(long)]
        name: Option<String>,
        #[command(flatten)]
        write_opts: WriteOpts,
    },
    /// End the current recording session
    End {
        file: PathBuf,
        #[command(flatten)]
        write_opts: WriteOpts,
    },
    /// List completed sessions
    List {
        file: PathBuf,
        #[arg(long)]
        json: bool,
        #[command(flatten)]
        write_opts: WriteOpts,
    },
    /// View a specific session
    View {
        file: PathBuf,
        #[arg(long = "session-id")]
        session_id: String,
        #[arg(long)]
        json: bool,
        #[command(flatten)]
        write_opts: WriteOpts,
    },
    /// Create a checkpoint in the current session
    Checkpoint {
        file: PathBuf,
        #[command(flatten)]
        write_opts: WriteOpts,
    },
    /// Delete a completed session
    Delete {
        file: PathBuf,
        #[arg(long = "session-id")]
        session_id: String,
        #[command(flatten)]
        write_opts: WriteOpts,
    },
}

pub fn run(args: SessionArgs) -> Result<()> {
    match args.command {
        SessionCommand::Start {
            file,
            name,
            write_opts,
        } => {
            let mut mem = crate::common::open_memory_rw(&file, &write_opts)?;
            // Load any existing active session
            mem.load_active_session()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            let session_id = mem
                .start_session(name.clone(), None)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            // Save sidecar so session persists across CLI invocations
            mem.save_active_session()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            println!("Started session: {session_id}");
            if let Some(n) = name {
                println!("  Name: {n}");
            }
            Ok(())
        }
        SessionCommand::End { file, write_opts } => {
            let mut mem = crate::common::open_memory_rw(&file, &write_opts)?;
            mem.load_active_session()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            let session = mem
                .end_session()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            // Persist and clear sidecar
            mem.save_replay_sessions()
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            mem.clear_active_session_file()
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            mem.commit().map_err(|e| anyhow::anyhow!("{e}"))?;

            println!("Ended session: {}", session.session_id);
            println!("  Actions recorded: {}", session.actions.len());
            Ok(())
        }
        SessionCommand::List {
            file,
            json,
            write_opts,
        } => {
            let mut mem = crate::common::open_memory_rw(&file, &write_opts)?;
            mem.load_replay_sessions()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            let sessions = mem.list_sessions();

            if json {
                println!("{}", serde_json::to_string_pretty(&sessions)?);
            } else {
                if sessions.is_empty() {
                    println!("No completed sessions found.");
                } else {
                    println!(
                        "{:<38} {:<20} {:<8}",
                        "Session ID", "Name", "Actions"
                    );
                    println!("{}", "-".repeat(70));
                    for s in &sessions {
                        println!(
                            "{:<38} {:<20} {:<8}",
                            s.session_id,
                            s.name.as_deref().unwrap_or("(unnamed)"),
                            s.action_count,
                        );
                    }
                }
            }

            // Also show active session if any
            mem.load_active_session()
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            if let Some(id) = mem.active_session_id() {
                println!("\nActive session: {id}");
            }
            Ok(())
        }
        SessionCommand::View {
            file,
            session_id,
            json,
            write_opts,
        } => {
            let mut mem = crate::common::open_memory_rw(&file, &write_opts)?;
            mem.load_replay_sessions()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            let uuid = session_id
                .parse::<uuid::Uuid>()
                .map_err(|e| anyhow::anyhow!("Invalid session ID: {e}"))?;

            match mem.get_session(uuid) {
                Some(session) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(session)?);
                    } else {
                        println!("Session: {}", session.session_id);
                        println!(
                            "  Name: {}",
                            session.name.as_deref().unwrap_or("(unnamed)")
                        );
                        println!("  Actions: {}", session.actions.len());
                        for (i, action) in session.actions.iter().enumerate() {
                            println!("  [{}] {:?}", i + 1, action.action_type);
                        }
                    }
                }
                None => {
                    eprintln!("Session {session_id} not found.");
                }
            }
            Ok(())
        }
        SessionCommand::Checkpoint { file, write_opts } => {
            let mut mem = crate::common::open_memory_rw(&file, &write_opts)?;
            mem.load_active_session()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            let checkpoint_id = mem
                .create_checkpoint()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            mem.save_active_session()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            println!("Created checkpoint: {checkpoint_id}");
            Ok(())
        }
        SessionCommand::Delete {
            file,
            session_id,
            write_opts,
        } => {
            let mut mem = crate::common::open_memory_rw(&file, &write_opts)?;
            mem.load_replay_sessions()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            let uuid = session_id
                .parse::<uuid::Uuid>()
                .map_err(|e| anyhow::anyhow!("Invalid session ID: {e}"))?;

            mem.delete_session(uuid)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            mem.save_replay_sessions()
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            mem.commit().map_err(|e| anyhow::anyhow!("{e}"))?;

            println!("Deleted session: {session_id}");
            Ok(())
        }
    }
}
