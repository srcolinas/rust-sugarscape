use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{SimulationConfig, World, Writer};

/// Deserialize simulation config from YAML text (same rules as file-based config).
pub fn parse_config_yaml(yaml: &str) -> Result<SimulationConfig> {
    let de = serde_yaml::Deserializer::from_reader(Cursor::new(yaml.as_bytes()));
    let config = serde_yaml::with::singleton_map_recursive::deserialize(de)?;
    Ok(config)
}

/// Read and deserialize simulation config from a YAML file.
pub fn read_config(path: impl AsRef<Path>) -> Result<SimulationConfig> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    let de = serde_yaml::Deserializer::from_reader(reader);
    let config = serde_yaml::with::singleton_map_recursive::deserialize(de)?;
    Ok(config)
}

/// Parquet path with the same basename as the config file, `.parquet` extension.
pub fn output_parquet_for_config_path(config_path: impl AsRef<Path>) -> PathBuf {
    config_path.as_ref().with_extension("parquet")
}

/// Run the simulation and write per-step rows to Parquet.
pub fn run_simulation(
    config: SimulationConfig,
    output: impl AsRef<Path>,
    buffer_limit: usize,
) -> Result<()> {
    let output = output.as_ref().to_path_buf();
    let mut world = World::new(&config.world, &config.agents);
    let mut writer = Writer::new(output, buffer_limit)?;
    for step in 0..config.run.iterations {
        let state = world.step();
        writer.add(step, &state)?;
    }

    writer.close()?;
    Ok(())
}
