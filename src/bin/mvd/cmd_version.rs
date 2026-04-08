use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct VersionArgs {}

pub fn run(_args: VersionArgs) -> Result<()> {
    println!("mvd {} (memvid-core {})", memvid_core::MEMVID_CORE_VERSION, memvid_core::MEMVID_CORE_VERSION);
    println!("Features:");
    println!("  lex:               {}", cfg!(feature = "lex"));
    println!("  vec:               {}", cfg!(feature = "vec"));
    println!("  clip:              {}", cfg!(feature = "clip"));
    println!("  whisper:           {}", cfg!(feature = "whisper"));
    println!("  encryption:        {}", cfg!(feature = "encryption"));
    println!("  replay:            {}", cfg!(feature = "replay"));
    println!("  parallel_segments: {}", cfg!(feature = "parallel_segments"));
    println!("  temporal_track:    {}", cfg!(feature = "temporal_track"));
    println!("  api_embed:         {}", cfg!(feature = "api_embed"));
    println!("  pdf_extract:       {}", cfg!(feature = "pdf_extract"));
    println!("  symspell_cleanup:  {}", cfg!(feature = "symspell_cleanup"));
    println!("  simd:              {}", cfg!(feature = "simd"));
    Ok(())
}
