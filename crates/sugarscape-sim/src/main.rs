use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use sugarscape_sim::{SimulationConfig, World};

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
    #[arg(long, value_name = "FILE")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = read_config(&cli.config)
        .with_context(|| format!("Failed to read config from {}", cli.config.display()))?;
    run_simulation(config);
    Ok(())
}

fn read_config(path: &PathBuf) -> Result<SimulationConfig> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let de = serde_yaml::Deserializer::from_reader(reader);
    let config = serde_yaml::with::singleton_map_recursive::deserialize(de)?;
    Ok(config)
}

pub fn run_simulation(config: SimulationConfig) {
    let world = World::new(&config.world);
    world.populate(&config.agents);
    for _ in 0..config.run.iterations {
        world.step();
    }
}
