use anyhow::Result;
use clap::Args;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Args)]
pub struct StatsArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub json: bool,
    /// Break down frame counts by repository, branch, and harness.
    #[arg(long = "by-repo")]
    pub by_repo: bool,
}

pub fn run(args: StatsArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_ro(&args.file)?;
    let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;

    let breakdown = if args.by_repo {
        Some(compute_breakdown(&mut mem)?)
    } else {
        None
    };

    if args.json {
        let mut value = serde_json::to_value(&stats)?;
        if let Some(b) = &breakdown {
            value["by_repo"] = serde_json::to_value(&b.by_repo)?;
            value["by_branch"] = serde_json::to_value(&b.by_branch)?;
            value["by_harness"] = serde_json::to_value(&b.by_harness)?;
        }
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        println!("Memory: {}", args.file.display());
        println!("  Total frames:       {}", stats.frame_count);
        println!("  Active frames:      {}", stats.active_frame_count);
        println!("  File size:          {}", crate::common::format_bytes(stats.size_bytes));
        println!("  Payload bytes:      {}", crate::common::format_bytes(stats.payload_bytes));
        println!("  Logical bytes:      {}", crate::common::format_bytes(stats.logical_bytes));
        println!("  Compression ratio:  {:.1}%", stats.compression_ratio_percent);
        println!("  Space savings:      {:.1}%", stats.savings_percent);
        println!("  Capacity:           {}", crate::common::format_bytes(stats.capacity_bytes));
        println!("  Remaining capacity: {}", crate::common::format_bytes(stats.remaining_capacity_bytes));
        println!("  Utilisation:        {:.1}%", stats.storage_utilisation_percent);

        println!("  Lex index:          {}", stats.has_lex_index);
        println!("  Vec index:          {}", stats.has_vec_index);
        println!("  Vectors:            {}", stats.vector_count);
        println!("  CLIP images:        {}", stats.clip_image_count);
        println!("  Time index:         {}", stats.has_time_index);
        println!("  WAL bytes:          {}", crate::common::format_bytes(stats.wal_bytes));

        if let Some(b) = &breakdown {
            println!("\nBy repository:");
            print_counts(&b.by_repo);
            println!("\nBy branch:");
            print_counts(&b.by_branch);
            println!("\nBy harness:");
            print_counts(&b.by_harness);
        }
    }
    Ok(())
}

struct Breakdown {
    by_repo: BTreeMap<String, u64>,
    by_branch: BTreeMap<String, u64>,
    by_harness: BTreeMap<String, u64>,
}

fn compute_breakdown(mem: &mut memvid_core::Memvid) -> Result<Breakdown> {
    let query = memvid_core::TimelineQuery::builder().no_limit().build();
    let entries = mem.timeline(query).map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut by_repo = BTreeMap::new();
    let mut by_branch = BTreeMap::new();
    let mut by_harness = BTreeMap::new();

    for e in &entries {
        let Ok(frame) = mem.frame_by_id(e.frame_id) else { continue };
        let m = &frame.extra_metadata;
        bump(&mut by_repo, m.get("repo"));
        bump(&mut by_branch, m.get("branch"));
        bump(&mut by_harness, m.get("harness"));
    }
    Ok(Breakdown { by_repo, by_branch, by_harness })
}

fn bump(map: &mut BTreeMap<String, u64>, value: Option<&String>) {
    let key = value.cloned().unwrap_or_else(|| "<unscoped>".to_string());
    *map.entry(key).or_insert(0) += 1;
}

fn print_counts(map: &BTreeMap<String, u64>) {
    if map.is_empty() {
        println!("  (no frames)");
        return;
    }
    let mut rows: Vec<(&String, &u64)> = map.iter().collect();
    rows.sort_by(|a, b| b.1.cmp(a.1));
    let max_label = rows.iter().map(|r| r.0.len()).max().unwrap_or(0);
    for (label, count) in rows {
        println!("  {label:<max_label$}  {count:>6}");
    }
}
