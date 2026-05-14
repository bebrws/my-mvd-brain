use anyhow::{Context, Result};
use clap::Args;
use memvid_core::Memvid;
use std::path::PathBuf;

/// Resolve the canonical memory-file path used by harness integrations and
/// optionally create it on disk.
///
/// Resolution priority:
///   1. `$MVD_FILE` environment variable, if set and non-empty
///   2. `$HOME/mvd.mv2`, if it exists (shared global brain)
///   3. `./mvd/mvd.mv2` (per-project fallback)
///
/// Without `--ensure` this is a pure lookup: it prints the path that *would*
/// be used, whether or not the file exists, and always exits 0 on success.
/// With `--ensure`, the file (and its parent directory) is created if missing
/// — so consumers can rely on the printed path being immediately openable.
#[derive(Args)]
pub struct ResolveArgs {
    /// Create the file (and parent directory) if it doesn't already exist.
    #[arg(long)]
    pub ensure: bool,

    /// Emit a JSON object with path, source, and existence metadata instead
    /// of a bare path.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    EnvVar,
    Global,
    Local,
}

impl Source {
    fn as_str(self) -> &'static str {
        match self {
            Source::EnvVar => "env",
            Source::Global => "global",
            Source::Local => "local",
        }
    }
}

pub fn run(args: ResolveArgs) -> Result<()> {
    let (path, source) = resolve();
    let existed_before = path.exists();

    let path = if args.ensure && !existed_before {
        ensure_file(&path)?
    } else {
        path
    };

    // Produce an absolute path for downstream `$(mvd resolve)` consumers, so
    // commands don't break when cwd changes mid-script.
    let absolute = absolutize(&path);
    let exists = absolute.exists();
    let created = args.ensure && !existed_before && exists;

    if args.json {
        let obj = serde_json::json!({
            "path":    absolute.display().to_string(),
            "source":  source.as_str(),
            "exists":  exists,
            "created": created,
        });
        println!("{}", serde_json::to_string(&obj)?);
    } else {
        println!("{}", absolute.display());
    }
    Ok(())
}

fn resolve() -> (PathBuf, Source) {
    if let Ok(env) = std::env::var("MVD_FILE") {
        if !env.is_empty() {
            return (PathBuf::from(env), Source::EnvVar);
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        let global = PathBuf::from(home).join("mvd.mv2");
        if global.exists() {
            return (global, Source::Global);
        }
    }
    (PathBuf::from("./mvd/mvd.mv2"), Source::Local)
}

/// Create the parent directory and the capsule file. Returns the (possibly
/// unchanged) path on success.
fn ensure_file(path: &std::path::Path) -> Result<PathBuf> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent: {}", parent.display()))?;
        }
    }
    // Memvid::create writes the empty capsule. We discard the handle — the
    // caller only needs the file to exist.
    let _ = Memvid::create(path)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("Failed to create memory: {}", path.display()))?;
    Ok(path.to_path_buf())
}

/// Make `path` absolute. Uses `canonicalize` when the file exists (so symlinks
/// are resolved), otherwise prefixes with `current_dir()` after stripping
/// any leading `./` so the printed path doesn't have a noisy `/./` segment.
fn absolutize(path: &std::path::Path) -> PathBuf {
    if let Ok(c) = std::fs::canonicalize(path) {
        return c;
    }
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let stripped: PathBuf = path
        .components()
        .filter(|c| !matches!(c, std::path::Component::CurDir))
        .collect();
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(stripped),
        Err(_) => path.to_path_buf(),
    }
}
