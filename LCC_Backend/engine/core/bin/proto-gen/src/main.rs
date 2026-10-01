//! lcc-proto-gen — emits a list of `.proto` files (CI helper).
//!
//! Real proto generation is delegated to `buf generate`.
//! This binary is a lightweight stub that lists proto files for the CI to verify.

use clap::Parser;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(about = "List proto files for CI")]
struct Cli {
    #[arg(long, default_value = "proto")]
    root: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let mut count = 0;
    for entry in WalkDir::new(&cli.root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("proto") {
            println!("{}", path.display());
            count += 1;
        }
    }
    tracing::info!(count, "found proto files");
    Ok(())
}
