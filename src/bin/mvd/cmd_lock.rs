use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct LockArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub password: Option<String>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub force: bool,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: LockArgs) -> Result<()> {
    #[cfg(feature = "encryption")]
    {
        let password = if let Some(ref pw) = args.password {
            pw.clone()
        } else {
            eprintln!("Enter password: ");
            let mut pw = String::new();
            std::io::stdin().read_line(&mut pw)?;
            pw.trim().to_string()
        };

        let out = args.out.as_deref();
        let result = memvid_core::encryption::lock_file(&args.file, out, password.as_bytes())
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        if args.json {
            println!("{{\"encrypted\":true,\"output\":\"{}\"}}", result.display());
        } else {
            println!("Encrypted {} → {}", args.file.display(), result.display());
        }
        Ok(())
    }
    #[cfg(not(feature = "encryption"))]
    {
        anyhow::bail!("Encryption feature is not enabled. Build with --features encryption");
    }
}
