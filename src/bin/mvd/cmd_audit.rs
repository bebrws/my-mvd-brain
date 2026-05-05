use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct AuditArgs {
    pub file: PathBuf,
    /// Question to audit retrieval for
    #[arg(long, short = 'q')]
    pub question: Option<String>,
    /// Include source snippets in the report
    #[arg(long)]
    pub snippets: bool,
    /// Output as markdown instead of plain text
    #[arg(long)]
    pub markdown: bool,
    /// Write report to a file
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
    /// Number of results to retrieve
    #[arg(long, default_value = "10")]
    pub top_k: usize,
}

pub fn run(args: AuditArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_ro(&args.file)?;

    // If no question is provided, show basic file stats as a health audit
    let question = match &args.question {
        Some(q) => q.clone(),
        None => {
            let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;
            let report = format!(
                "Audit Report for {}\n\
                 Frames: {}\n\
                 Active: {}\n\
                 Size: {}\n\
                 Lex index: {}\n\
                 Vec index: {}\n\
                 Time index: {}",
                args.file.display(),
                stats.frame_count,
                stats.active_frame_count,
                crate::common::format_bytes(stats.size_bytes),
                stats.has_lex_index,
                stats.has_vec_index,
                stats.has_time_index,
            );
            if let Some(ref out_path) = args.out {
                std::fs::write(out_path, &report)?;
                println!("Audit report written to {}", out_path.display());
            } else {
                println!("{report}");
            }
            return Ok(());
        }
    };

    // Use the full mem.audit() API for question-based provenance reports
    let options = memvid_core::types::audit::AuditOptions {
        top_k: Some(args.top_k),
        include_snippets: args.snippets,
        ..Default::default()
    };

    let report = mem
        .audit::<dyn memvid_core::VecEmbedder>(&question, Some(options), None)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let output = if args.json {
        serde_json::to_string_pretty(&report)?
    } else if args.markdown {
        report.to_markdown()
    } else {
        report.to_text()
    };

    if let Some(ref out_path) = args.out {
        std::fs::write(out_path, &output)?;
        println!("Audit report written to {}", out_path.display());
    } else {
        println!("{output}");
    }
    Ok(())
}
