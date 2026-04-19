use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use sugarscape_sim::{SimulationConfig, World, Writer};

#[derive(Parser, Debug)]
#[command(
    name = "sugarscape-sim",
    about = "Run sugarscape simulation and write Parquet stats"
)]
struct Cli {
    /// Path to the simulation YAML config
    #[arg(long, value_name = "FILE")]
    config: PathBuf,

    /// Output Parquet path (per-iteration aggregate stats)
    #[arg(long, value_name = "FILE", default_value = "run.parquet")]
    output: PathBuf,

    /// Buffer limit for writing Parquet
    #[arg(long, value_name = "N", default_value = "1000")]
    buffer_limit: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = read_config(&cli.config)
        .with_context(|| format!("Failed to read config from {}", cli.config.display()))?;
    run_simulation(config, cli.output, cli.buffer_limit)?;
    Ok(())
}

fn read_config(path: &PathBuf) -> Result<SimulationConfig> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let de = serde_yaml::Deserializer::from_reader(reader);
    let config = serde_yaml::with::singleton_map_recursive::deserialize(de)?;
    Ok(config)
}

fn run_simulation(config: SimulationConfig, output: PathBuf, buffer_limit: usize) -> Result<()> {
    let mut world = World::new(&config.world, &config.agents);
    let mut writer = Writer::new(output, buffer_limit)?;
    for step in 0..config.run.iterations {
        let state = world.step();
        writer.add(step, &state)?;
    }

    writer.close()?;
    Ok(())
}
