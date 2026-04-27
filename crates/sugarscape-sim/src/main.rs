use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use sugarscape_sim::{output_parquet_for_config_path, read_config, run_simulation};

#[derive(Parser, Debug)]
#[command(
    name = "sugarscape-sim",
    about = "Run sugarscape simulation and write Parquet stats"
)]
struct Cli {
    /// Path to the simulation YAML config
    #[arg(long, value_name = "FILE")]
    config: PathBuf,

    /// Output Parquet path (default: same path as config with `.parquet` extension)
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Buffer limit for writing Parquet
    #[arg(long, value_name = "N", default_value = "1000")]
    buffer_limit: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = read_config(&cli.config)
        .with_context(|| format!("Failed to read config from {}", cli.config.display()))?;
    let output = cli
        .output
        .unwrap_or_else(|| output_parquet_for_config_path(&cli.config));
    run_simulation(config, output, cli.buffer_limit)?;
    Ok(())
}
