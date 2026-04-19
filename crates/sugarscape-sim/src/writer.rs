use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{Float32Array, UInt32Array, UInt8Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::errors::ParquetError;
use std::io;

use crate::config::CellPosition;
use crate::world::{AgentState, CellLevel};

#[derive(Debug, thiserror::Error)]
pub enum WriterError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Parquet(#[from] ParquetError),
}

pub struct Writer {
    writer: ArrowWriter<File>,
    schema: Arc<Schema>,

    step_buffer: Vec<u32>,
    row_buffer: Vec<u8>,
    col_buffer: Vec<u8>,
    level_buffer: Vec<f32>,
    wealth_buffer: Vec<f32>,
    age_buffer: Vec<u32>,
    buffer_limit: usize,
}

impl Writer {
    pub fn new(file: PathBuf, buffer_limit: usize) -> Result<Writer, WriterError> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("step", DataType::UInt32, false),
            Field::new("row", DataType::UInt8, false),
            Field::new("col", DataType::UInt8, false),
            Field::new("level", DataType::Float32, false),
            Field::new("wealth", DataType::Float32, false),
            Field::new("age", DataType::UInt32, false),
        ]));

        let f = File::create(file)?;
        let writer = ArrowWriter::try_new(f, schema.clone(), None)?;

        Ok(Writer {
            writer,
            schema,
            step_buffer: Vec::with_capacity(buffer_limit),
            row_buffer: Vec::with_capacity(buffer_limit),
            col_buffer: Vec::with_capacity(buffer_limit),
            level_buffer: Vec::with_capacity(buffer_limit),
            wealth_buffer: Vec::with_capacity(buffer_limit),
            age_buffer: Vec::with_capacity(buffer_limit),
            buffer_limit,
        })
    }

    pub fn add(
        &mut self,
        step: u32,
        state: &HashMap<CellPosition, (CellLevel, AgentState)>,
    ) -> Result<(), WriterError> {
        // 1. Flatten the HashMap into the buffers
        for (pos, (level, agent)) in state.iter() {
            self.step_buffer.push(step);
            self.row_buffer.push(pos.row);
            self.col_buffer.push(pos.col);
            self.level_buffer.push(level.0);
            self.wealth_buffer.push(agent.1);
            self.age_buffer.push(agent.0);
        }

        // 2. Check if we should flush to Parquet
        if self.step_buffer.len() >= self.buffer_limit {
            self.flush_buffer()?;
        }

        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<(), WriterError> {
        if self.step_buffer.is_empty() {
            return Ok(());
        }

        // Create Arrow Arrays from buffers
        let batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.step_buffer))),
                Arc::new(UInt8Array::from(std::mem::take(&mut self.row_buffer))),
                Arc::new(UInt8Array::from(std::mem::take(&mut self.col_buffer))),
                Arc::new(Float32Array::from(std::mem::take(&mut self.level_buffer))),
                Arc::new(Float32Array::from(std::mem::take(&mut self.wealth_buffer))),
                Arc::new(UInt32Array::from(std::mem::take(&mut self.age_buffer))),
            ],
        )
        .map_err(|e| WriterError::Parquet(ParquetError::General(e.to_string())))?; // Simplification for example

        self.writer.write(&batch)?;

        self.step_buffer.reserve(self.buffer_limit);
        self.row_buffer.reserve(self.buffer_limit);
        self.col_buffer.reserve(self.buffer_limit);
        self.level_buffer.reserve(self.buffer_limit);
        self.wealth_buffer.reserve(self.buffer_limit);
        self.age_buffer.reserve(self.buffer_limit);

        Ok(())
    }

    pub fn close(mut self) -> Result<(), WriterError> {
        self.flush_buffer()?; // Don't lose the trailing data!
        self.writer.close()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use tempfile::tempdir;

    #[test]
    fn writer_adds_data_to_buffer() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.parquet");

        let mut writer = Writer::new(path.clone(), 1000).unwrap();
        writer
            .add(
                0,
                &HashMap::from([(
                    CellPosition { row: 0, col: 0 },
                    (CellLevel(0.0), AgentState(0, 0.0)),
                )]),
            )
            .unwrap();
        writer.close().unwrap();

        let file = File::open(&path).unwrap();
        let mut reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();
        let batch = reader.next().unwrap().unwrap();
        assert_eq!(batch.num_rows(), 1);

        let step = batch
            .column(0)
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();
        let row = batch
            .column(1)
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        let col = batch
            .column(2)
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        let level = batch
            .column(3)
            .as_any()
            .downcast_ref::<Float32Array>()
            .unwrap();
        let wealth = batch
            .column(4)
            .as_any()
            .downcast_ref::<Float32Array>()
            .unwrap();
        let age = batch
            .column(5)
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();

        assert_eq!(step.value(0), 0);
        assert_eq!(row.value(0), 0);
        assert_eq!(col.value(0), 0);
        assert_eq!(level.value(0), 0.0);
        assert_eq!(wealth.value(0), 0.0);
        assert_eq!(age.value(0), 0);
    }
}
