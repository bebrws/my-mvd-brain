//! Append-only usage log for `mvd` invocations.
//!
//! Stored at `$HOME/.mvd/usage.jsonl`, one JSON object per line. Captures
//! who is using `mvd` (harness / git context) and which commands they run.
//! Reads, writes, and reads-with-no-results all leave a trace, so `mvd usage`
//! can answer "what's been calling me" even when nothing was committed.
//!
//! Privacy: flag *names* are recorded, but flag *values* and the query string
//! are never logged.
//!
//! Failures are silent — the logger never blocks a CLI command.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use serde::Serialize;

use crate::git_context::GitContext;
use crate::harness::Harness;

const ROTATE_AT_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Serialize)]
struct UsageEntry<'a> {
    ts: String,
    command: &'a str,
    exit_code: i32,
    duration_ms: u128,
    harness: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    harness_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    flags: Vec<String>,
    cwd: Option<String>,
}

pub fn enabled() -> bool {
    !matches!(
        std::env::var("MVD_USAGE_LOG").as_deref(),
        Ok("0") | Ok("false") | Ok("off") | Ok("no")
    )
}

pub fn log_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".mvd").join("usage.jsonl"))
}

/// Record one CLI invocation. Flags must be flag *names only*, no values.
pub fn record(
    command: &str,
    exit_code: i32,
    duration_ms: u128,
    flags: Vec<String>,
) {
    if !enabled() {
        return;
    }
    let Some(path) = log_path() else { return; };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    rotate_if_needed(&path);

    let harness = Harness::detect();
    let git = GitContext::detect();
    let entry = UsageEntry {
        ts: now_iso(),
        command,
        exit_code,
        duration_ms,
        harness: &harness.name,
        harness_version: harness.version,
        repo: git.repo,
        branch: git.branch,
        flags,
        cwd: std::env::current_dir().ok().and_then(|p| p.to_str().map(|s| s.to_string())),
    };

    let Ok(line) = serde_json::to_string(&entry) else { return; };
    let mut file = match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let _ = writeln!(file, "{line}");
}

fn rotate_if_needed(path: &std::path::Path) {
    let Ok(meta) = fs::metadata(path) else { return; };
    if meta.len() < ROTATE_AT_BYTES {
        return;
    }
    let rotated = path.with_extension("jsonl.1");
    let _ = fs::rename(path, &rotated);
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_unix_iso(secs as i64)
}

/// Minimal UTC ISO-8601 formatter (`YYYY-MM-DDTHH:MM:SSZ`) for an epoch second.
/// Avoids pulling in `chrono` just for the logger.
fn format_unix_iso(secs: i64) -> String {
    let mut t = secs;
    let sec = (t.rem_euclid(60)) as u32; t = t.div_euclid(60);
    let min = (t.rem_euclid(60)) as u32; t = t.div_euclid(60);
    let hour = (t.rem_euclid(24)) as u32; t = t.div_euclid(24);
    // t is now days since 1970-01-01.
    let (y, m, d) = days_to_ymd(t);
    format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn days_to_ymd(mut days: i64) -> (i32, u32, u32) {
    // Howard Hinnant's days-from-civil algorithm.
    days += 719_468;
    let era = if days >= 0 { days / 146_097 } else { (days - 146_096) / 146_097 };
    let doe = (days - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}
