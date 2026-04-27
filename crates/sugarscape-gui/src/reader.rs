use std::fs::File;
use std::path::PathBuf;

use arrow::array::{Array, Float32Array, UInt32Array, UInt8Array};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};
use parquet::errors::ParquetError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Parquet(#[from] ParquetError),
    #[error(transparent)]
    Arrow(#[from] ArrowError),
    #[error("column {column}: expected {expected}")]
    ColumnDowncast {
        column: usize,
        expected: &'static str,
    },
}

pub type Step = u32;

#[derive(Debug, PartialEq, Default)]
pub struct StepData {
    pub rows: Vec<u8>,
    pub cols: Vec<u8>,
    pub levels: Vec<f32>,
    pub wealths: Vec<f32>,
    pub ages: Vec<u32>,
}

pub struct SimulationDataReader {
    reader: ParquetRecordBatchReader,
    current_step: Option<Step>,
    current_data: StepData,
    /// When a `next()` returns mid-batch, remaining rows stay here.
    pending_batch: Option<RecordBatch>,
    pending_index: usize,
}

impl SimulationDataReader {
    pub fn new(file: &PathBuf) -> Result<Self, ReaderError> {
        let file = File::open(file)?;
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)?.build()?;
        Ok(Self {
            reader,
            current_step: None,
            current_data: StepData::default(),
            pending_batch: None,
            pending_index: 0,
        })
    }

    fn load_next_batch(&mut self) -> Result<bool, ReaderError> {
        if self.pending_batch.is_some() {
            return Ok(true);
        }
        match self.reader.next() {
            None => Ok(false),
            Some(Err(e)) => Err(e.into()),
            Some(Ok(batch)) => {
                self.pending_batch = Some(batch);
                self.pending_index = 0;
                Ok(true)
            }
        }
    }

    fn flush_on_eof(&mut self) -> Option<Result<(Step, StepData), ReaderError>> {
        let step = self.current_step.take()?;
        let data = std::mem::take(&mut self.current_data);
        if data.rows.is_empty() {
            None
        } else {
            Some(Ok((step, data)))
        }
    }
}

impl Iterator for SimulationDataReader {
    type Item = Result<(Step, StepData), ReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !match self.load_next_batch() {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            } {
                return self.flush_on_eof();
            }

            let batch = self.pending_batch.as_ref().unwrap();
            let steps = match downcast_column::<UInt32Array>(batch, 0, "UInt32Array (step)") {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let rows = match downcast_column::<UInt8Array>(batch, 1, "UInt8Array (row)") {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let cols = match downcast_column::<UInt8Array>(batch, 2, "UInt8Array (col)") {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let levels = match downcast_column::<Float32Array>(batch, 3, "Float32Array (level)") {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let wealths = match downcast_column::<Float32Array>(batch, 4, "Float32Array (wealth)") {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let ages = match downcast_column::<UInt32Array>(batch, 5, "UInt32Array (age)") {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };

            let n = batch.num_rows();
            let start = self.pending_index;

            for i in start..n {
                let step = steps.value(i);

                match self.current_step {
                    None => {
                        self.current_step = Some(step);
                        push_row(&mut self.current_data, rows, cols, levels, wealths, ages, i);
                    }
                    Some(cur) if cur == step => {
                        push_row(&mut self.current_data, rows, cols, levels, wealths, ages, i);
                    }
                    Some(cur) => {
                        let finished_step = cur;
                        let finished_data = std::mem::take(&mut self.current_data);
                        self.current_step = Some(step);
                        push_row(&mut self.current_data, rows, cols, levels, wealths, ages, i);
                        self.pending_index = i + 1;
                        if self.pending_index >= n {
                            self.pending_batch = None;
                            self.pending_index = 0;
                        }
                        return Some(Ok((finished_step, finished_data)));
                    }
                }
            }

            self.pending_batch = None;
            self.pending_index = 0;
        }
    }
}

fn push_row(
    data: &mut StepData,
    rows: &UInt8Array,
    cols: &UInt8Array,
    levels: &Float32Array,
    wealths: &Float32Array,
    ages: &UInt32Array,
    i: usize,
) {
    data.rows.push(rows.value(i));
    data.cols.push(cols.value(i));
    data.levels.push(levels.value(i));
    data.wealths.push(wealths.value(i));
    data.ages.push(ages.value(i));
}

fn downcast_column<'a, T: Array + 'static>(
    batch: &'a RecordBatch,
    column: usize,
    expected: &'static str,
) -> Result<&'a T, ReaderError> {
    batch
        .column(column)
        .as_any()
        .downcast_ref::<T>()
        .ok_or(ReaderError::ColumnDowncast { column, expected })
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use arrow::datatypes::{DataType, Field, Schema};
    use parquet::arrow::arrow_writer::ArrowWriter;
    use tempfile::tempdir;

    use super::*;

    struct TestData {
        steps: UInt32Array,
        rows: UInt8Array,
        cols: UInt8Array,
        levels: Float32Array,
        wealths: Float32Array,
        ages: UInt32Array,
    }

    fn write_test_parquet(path: &Path, data: &TestData) {
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("step", DataType::UInt32, false),
                Field::new("row", DataType::UInt8, false),
                Field::new("col", DataType::UInt8, false),
                Field::new("level", DataType::Float32, false),
                Field::new("wealth", DataType::Float32, false),
                Field::new("age", DataType::UInt32, false),
            ])),
            vec![
                Arc::new(data.steps.clone()),
                Arc::new(data.rows.clone()),
                Arc::new(data.cols.clone()),
                Arc::new(data.levels.clone()),
                Arc::new(data.wealths.clone()),
                Arc::new(data.ages.clone()),
            ],
        )
        .unwrap();
        let file = File::create(path).unwrap();
        let mut writer = ArrowWriter::try_new(file, batch.schema(), None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
    }

    fn write_two_batches_same_schema(path: &Path, a: &TestData, b: &TestData) {
        let schema = Arc::new(Schema::new(vec![
            Field::new("step", DataType::UInt32, false),
            Field::new("row", DataType::UInt8, false),
            Field::new("col", DataType::UInt8, false),
            Field::new("level", DataType::Float32, false),
            Field::new("wealth", DataType::Float32, false),
            Field::new("age", DataType::UInt32, false),
        ]));
        let batch_a = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(a.steps.clone()),
                Arc::new(a.rows.clone()),
                Arc::new(a.cols.clone()),
                Arc::new(a.levels.clone()),
                Arc::new(a.wealths.clone()),
                Arc::new(a.ages.clone()),
            ],
        )
        .unwrap();
        let batch_b = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(b.steps.clone()),
                Arc::new(b.rows.clone()),
                Arc::new(b.cols.clone()),
                Arc::new(b.levels.clone()),
                Arc::new(b.wealths.clone()),
                Arc::new(b.ages.clone()),
            ],
        )
        .unwrap();
        let file = File::create(path).unwrap();
        let mut writer = ArrowWriter::try_new(file, batch_a.schema(), None).unwrap();
        writer.write(&batch_a).unwrap();
        writer.write(&batch_b).unwrap();
        writer.close().unwrap();
    }

    #[test]
    fn multiple_steps_in_one_batch() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("batch.parquet");
        write_test_parquet(
            &path,
            &TestData {
                steps: UInt32Array::from(vec![1, 2]),
                rows: UInt8Array::from(vec![0, 9]),
                cols: UInt8Array::from(vec![3, 4]),
                levels: Float32Array::from(vec![0.5, 2.0]),
                wealths: Float32Array::from(vec![1.5, 3.0]),
                ages: UInt32Array::from(vec![10, 20]),
            },
        );
        let reader = SimulationDataReader::new(&path).unwrap();
        let mut reader = reader.into_iter();
        let (step, data) = reader.next().unwrap().unwrap();

        assert_eq!(step, 1);
        assert_eq!(
            data,
            StepData {
                rows: vec![0],
                cols: vec![3],
                levels: vec![0.5],
                wealths: vec![1.5],
                ages: vec![10]
            }
        );

        let (step, data) = reader.next().unwrap().unwrap();
        assert_eq!(step, 2);
        assert_eq!(
            data,
            StepData {
                rows: vec![9],
                cols: vec![4],
                levels: vec![2.0],
                wealths: vec![3.0],
                ages: vec![20]
            }
        );
        assert!(reader.next().is_none());
    }

    #[test]
    fn same_step_spread_across_multiple_batches() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("batch.parquet");
        write_two_batches_same_schema(
            &path,
            &TestData {
                steps: UInt32Array::from(vec![1]),
                rows: UInt8Array::from(vec![0]),
                cols: UInt8Array::from(vec![3]),
                levels: Float32Array::from(vec![0.5]),
                wealths: Float32Array::from(vec![1.5]),
                ages: UInt32Array::from(vec![10]),
            },
            &TestData {
                steps: UInt32Array::from(vec![1]),
                rows: UInt8Array::from(vec![1]),
                cols: UInt8Array::from(vec![5]),
                levels: Float32Array::from(vec![1.0]),
                wealths: Float32Array::from(vec![2.0]),
                ages: UInt32Array::from(vec![11]),
            },
        );
        let reader = SimulationDataReader::new(&path).unwrap();
        let mut reader = reader.into_iter();
        let (step, data) = reader.next().unwrap().unwrap();
        assert_eq!(step, 1);
        assert_eq!(
            data,
            StepData {
                rows: vec![0, 1],
                cols: vec![3, 5],
                levels: vec![0.5, 1.0],
                wealths: vec![1.5, 2.0],
                ages: vec![10, 11]
            }
        );
        assert!(reader.next().is_none());
    }
}
