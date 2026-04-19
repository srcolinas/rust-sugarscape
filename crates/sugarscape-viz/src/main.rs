use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use arrow::array::{Float32Array, UInt32Array, UInt8Array};
use arrow::record_batch::RecordBatch;
use clap::Parser;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

#[derive(Parser, Debug)]
#[command(name = "sugarscape-viz", about = "Read simulation Parquet stats (visualization hook)")]
struct Cli {
    /// Parquet file produced by sugarscape-sim
    #[arg(long, value_name = "FILE")]
    input: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let file = File::open(&cli.input)
        .with_context(|| format!("open {}", cli.input.display()))?;

    let mut reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .context("read parquet metadata")?
        .build()
        .context("build parquet reader")?;

    while let Some(batch) = reader.next() {
        let batch = batch.context("read record batch")?;
        print_batch_rows(&batch)?;
    }

    Ok(())
}

/// Layout matches `sugarscape-sim` Parquet writer: step, row, col, level, wealth, age.
fn print_batch_rows(batch: &RecordBatch) -> Result<()> {
    if batch.num_columns() != 6 {
        anyhow::bail!(
            "expected 6 columns (step, row, col, level, wealth, age), got {}",
            batch.num_columns()
        );
    }

    let step = batch
        .column(0)
        .as_any()
        .downcast_ref::<UInt32Array>()
        .context("column 0 (step) must be UInt32")?;
    let row = batch
        .column(1)
        .as_any()
        .downcast_ref::<UInt8Array>()
        .context("column 1 (row) must be UInt8")?;
    let col = batch
        .column(2)
        .as_any()
        .downcast_ref::<UInt8Array>()
        .context("column 2 (col) must be UInt8")?;
    let level = batch
        .column(3)
        .as_any()
        .downcast_ref::<Float32Array>()
        .context("column 3 (level) must be Float32")?;
    let wealth = batch
        .column(4)
        .as_any()
        .downcast_ref::<Float32Array>()
        .context("column 4 (wealth) must be Float32")?;
    let age = batch
        .column(5)
        .as_any()
        .downcast_ref::<UInt32Array>()
        .context("column 5 (age) must be UInt32")?;

    for i in 0..batch.num_rows() {
        println!(
            "step={} row={} col={} level={} wealth={} age={}",
            step.value(i),
            row.value(i),
            col.value(i),
            level.value(i),
            wealth.value(i),
            age.value(i),
        );
    }

    Ok(())
}

