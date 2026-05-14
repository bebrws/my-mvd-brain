//! `mvd usage` — summarize how mvd has been used (which harness, which
//! commands, which repos). Reads `$HOME/.mvd/usage.jsonl` plus, optionally,
//! frame `extra_metadata` from the capsule.

use anyhow::{Context, Result};
use clap::Args;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Args)]
pub struct UsageArgs {
    /// Optional `.mv2` file to derive write-side stats from when `--frames` is set.
    pub file: Option<PathBuf>,
    /// Group-by axis: harness | command | repo | branch | day.
    #[arg(long, default_value = "harness", value_name = "AXIS")]
    pub by: String,
    /// Time window: 7d, 30d, 90d, all.
    #[arg(long, default_value = "30d", value_name = "WIN")]
    pub since: String,
    /// Filter to one harness (e.g. cursor, claude-code).
    #[arg(long, value_name = "NAME")]
    pub harness: Option<String>,
    /// Filter to one repository.
    #[arg(long, value_name = "ID")]
    pub repo: Option<String>,
    /// Filter to one branch.
    #[arg(long, value_name = "NAME")]
    pub branch: Option<String>,
    /// Use frame metadata from the capsule (durable, writes only).
    #[arg(long)]
    pub frames: bool,
    /// JSON output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Deserialize, Debug)]
struct UsageEntry {
    ts: String,
    command: String,
    #[allow(dead_code)]
    exit_code: i32,
    #[allow(dead_code)]
    duration_ms: u128,
    harness: String,
    #[serde(default)]
    harness_version: Option<String>,
    #[serde(default)]
    repo: Option<String>,
    #[serde(default)]
    branch: Option<String>,
}

pub fn run(args: UsageArgs) -> Result<()> {
    if args.frames {
        return run_from_frames(&args);
    }

    let path = crate::usage_log::log_path()
        .context("Cannot determine $HOME/.mvd/usage.jsonl path")?;
    if !path.exists() {
        if args.json {
            println!("{{\"entries\":0,\"reason\":\"no usage log yet\"}}");
        } else {
            println!("No usage log yet at {}", path.display());
            println!("(Run a few mvd commands first; logs are appended automatically.)");
        }
        return Ok(());
    }

    let cutoff = parse_since(&args.since);
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let mut entries: Vec<UsageEntry> = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(e) = serde_json::from_str::<UsageEntry>(line) else { continue };
        if let Some(ref h) = args.harness {
            if &e.harness != h { continue; }
        }
        if let Some(ref r) = args.repo {
            if e.repo.as_deref() != Some(r.as_str()) { continue; }
        }
        if let Some(ref b) = args.branch {
            if e.branch.as_deref() != Some(b.as_str()) { continue; }
        }
        if let Some(cutoff_ts) = cutoff {
            if iso_to_epoch(&e.ts).map(|t| t < cutoff_ts).unwrap_or(false) {
                continue;
            }
        }
        entries.push(e);
    }

    if args.json {
        let summary = summarize_json(&entries, &args.by);
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_text_summary(&entries, &args, &path);
    }
    Ok(())
}

fn run_from_frames(args: &UsageArgs) -> Result<()> {
    let _ = args;
    anyhow::bail!("--frames mode requires reading the capsule and is not yet implemented; use the default usage-log mode")
}

fn parse_since(s: &str) -> Option<i64> {
    let s = s.trim();
    if s == "all" || s.is_empty() { return None; }
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .ok()?;
    let (num_str, unit) = s.split_at(s.len().saturating_sub(1));
    let n: i64 = num_str.parse().ok()?;
    let secs = match unit {
        "d" => n * 86_400,
        "h" => n * 3_600,
        "w" => n * 7 * 86_400,
        _ => return None,
    };
    Some(now - secs)
}

/// Parse `YYYY-MM-DDTHH:MM:SSZ` into epoch seconds.
fn iso_to_epoch(ts: &str) -> Option<i64> {
    let bytes = ts.as_bytes();
    if bytes.len() < 20 { return None; }
    let parse2 = |i: usize| -> Option<i64> {
        std::str::from_utf8(&bytes[i..i+2]).ok()?.parse().ok()
    };
    let parse4 = |i: usize| -> Option<i64> {
        std::str::from_utf8(&bytes[i..i+4]).ok()?.parse().ok()
    };
    let year = parse4(0)?;
    let month = parse2(5)?;
    let day = parse2(8)?;
    let hour = parse2(11)?;
    let min = parse2(14)?;
    let sec = parse2(17)?;
    Some(ymd_to_epoch(year, month as u32, day as u32) + hour * 3600 + min * 60 + sec)
}

fn ymd_to_epoch(year: i64, month: u32, day: u32) -> i64 {
    // Howard Hinnant's days-from-civil.
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y / 400 } else { (y - 399) / 400 };
    let yoe = (y - era * 400) as u64;
    let m = month as u64;
    let m = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * m + 2) / 5 + day as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe as i64 - 719_468;
    days * 86_400
}

fn summarize_json(entries: &[UsageEntry], by: &str) -> serde_json::Value {
    let mut by_axis: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();

    for e in entries {
        let key = axis_value(e, by);
        *totals.entry(key.clone()).or_insert(0) += 1;
        *by_axis.entry(key).or_default().entry(e.command.clone()).or_insert(0) += 1;
    }

    serde_json::json!({
        "total_invocations": entries.len(),
        "by": by,
        "groups": totals,
        "groups_by_command": by_axis,
    })
}

fn axis_value(e: &UsageEntry, by: &str) -> String {
    match by {
        "harness" => e.harness.clone(),
        "command" => e.command.clone(),
        "repo" => e.repo.clone().unwrap_or_else(|| "<unscoped>".into()),
        "branch" => e.branch.clone().unwrap_or_else(|| "<unscoped>".into()),
        "day" => e.ts.get(..10).unwrap_or(&e.ts).to_string(),
        _ => e.harness.clone(),
    }
}

fn print_text_summary(entries: &[UsageEntry], args: &UsageArgs, path: &std::path::Path) {
    println!("mvd usage — last {} (log: {})", args.since, path.display());
    if entries.is_empty() {
        println!("  (no entries match the filters)");
        return;
    }

    let total = entries.len();
    println!("  {total} invocations\n");

    let mut by_axis: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    for e in entries {
        let key = axis_value(e, &args.by);
        *by_axis.entry(key).or_default().entry(e.command.clone()).or_insert(0) += 1;
    }

    println!("By {}:", args.by);
    let mut rows: Vec<(String, u64, BTreeMap<String, u64>)> = by_axis
        .into_iter()
        .map(|(k, cmds)| {
            let total = cmds.values().sum();
            (k, total, cmds)
        })
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1));

    let max_label = rows.iter().map(|r| r.0.len()).max().unwrap_or(0).max(8);
    for (label, count, cmds) in rows {
        let mut top: Vec<(String, u64)> = cmds.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        let top_str = top
            .iter()
            .take(8)
            .map(|(c, n)| format!("{c} {n}"))
            .collect::<Vec<_>>()
            .join(" · ");
        println!("  {label:<max_label$}  {count:>6}   {top_str}");
    }

    if args.by != "harness" {
        println!();
        println!("Harnesses seen: {}", distinct_harnesses(entries).join(", "));
    }
}

fn distinct_harnesses(entries: &[UsageEntry]) -> Vec<String> {
    let mut s: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for e in entries {
        match &e.harness_version {
            Some(v) if !v.is_empty() => s.insert(format!("{} ({v})", e.harness)),
            _ => s.insert(e.harness.clone()),
        };
    }
    s.into_iter().collect()
}
