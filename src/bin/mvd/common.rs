use std::path::Path;
use anyhow::{Context, Result};
use memvid_core::Memvid;

/// Shared write-lock options used by mutation commands (put, update, delete, etc.)
#[derive(clap::Args, Clone, Debug)]
pub struct WriteOpts {
    /// Maximum time to wait for an active writer before failing (ms)
    #[arg(long, default_value = "250")]
    pub lock_timeout: u64,

    /// Attempt a stale takeover if the recorded writer heartbeat has expired
    #[arg(long)]
    pub force: bool,
}

/// Open a memory file for writing, applying lock settings.
pub fn open_memory_rw(path: &Path, _write_opts: &WriteOpts) -> Result<Memvid> {
    Memvid::open(path)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("Failed to open memory: {}", path.display()))
}

/// Open a memory file for read-only access.
pub fn open_memory_ro(path: &Path) -> Result<Memvid> {
    Memvid::open_read_only(path)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("Failed to open memory: {}", path.display()))
}



/// Format bytes as a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Config directory for mvd CLI.
pub fn config_dir() -> Result<std::path::PathBuf> {
    let home = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .map_err(|_| anyhow::anyhow!("Cannot determine home directory"))?;
    Ok(home.join(".config").join("mvd"))
}

/// Config file path.
pub fn config_file_path() -> Result<std::path::PathBuf> {
    Ok(config_dir()?.join("config.json"))
}
