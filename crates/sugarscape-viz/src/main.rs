use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "sugarscape-viz", about = "Read simulation Parquet stats (visualization hook)")]
struct Cli {
    /// Parquet file produced by sugarscape-sim
    #[arg(long, value_name = "FILE")]
    input: PathBuf,
}

fn main() -> Result<()> {
    todo!()
}

