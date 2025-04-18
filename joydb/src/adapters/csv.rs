use crate::adapters::{Adapter, Partitioned, PartitionedAdapter};
use crate::{JoydbError, state::State};
use crate::{Model, Relation};
use csv::Writer;
use std::path::{Path, PathBuf};

use super::{FromPath, fs_utils};

/// A CSV adapter.
/// Stores every relation in a separate CSV file.
pub struct CsvAdapter {
    dir_path: PathBuf,
}

impl FromPath for CsvAdapter {
    fn from_path<P: AsRef<Path>>(dir_path: P) -> Self {
        Self::new(dir_path)
    }
}

impl CsvAdapter {
    pub fn new<P: AsRef<Path>>(dir_path: P) -> Self {
        Self {
            dir_path: dir_path.as_ref().to_path_buf(),
        }
    }

    fn relation_file_path<M: Model>(&self) -> PathBuf {
        self.dir_path.join(format!("{}.csv", M::model_name()))
    }
}

impl PartitionedAdapter for CsvAdapter {
    fn write_relation<M: Model>(&self, relation: &Relation<M>) -> Result<(), JoydbError> {
        let file_path = self.relation_file_path::<M>();
        write_relation_to_csv_file(relation, &file_path)
    }

    fn load_relation<M: Model>(&self) -> Result<Relation<M>, JoydbError> {
        let file_path = self.relation_file_path::<M>();
        if file_path.exists() {
            if !file_path.is_file() {
                // If the path exists but is not a file, then return an error
                Err(JoydbError::NotFile(file_path))
            } else {
                // Otherwise read the relation from the existing file
                let file_path = self.relation_file_path::<M>();
                load_relation_from_csv_file::<M>(&file_path)
            }
        } else {
            // If the file does not exist, create a new file with empty relation
            let empty_relation = Relation::<M>::default();
            self.write_relation(&empty_relation)?;
            Ok(empty_relation)
        }
    }

    fn load_state<S: State>(&self) -> Result<S, JoydbError> {
        if self.dir_path.exists() {
            if !self.dir_path.is_dir() {
                return Err(JoydbError::NotDirectory(self.dir_path.clone()));
            }
        } else {
            // Create a directory if it does not exist
            std::fs::create_dir_all(&self.dir_path)?;
        }

        S::load_with_partitioned_adapter(self)
    }
}

impl Adapter for CsvAdapter {
    type Target = Partitioned<Self>;
}

fn load_relation_from_csv_file<M: Model>(file_path: &PathBuf) -> Result<Relation<M>, JoydbError> {
    let mut reader =
        csv::Reader::from_path(file_path).map_err(|e| JoydbError::Deserialize(Box::new(e)))?;

    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: M = result.map_err(|e| JoydbError::Deserialize(Box::new(e)))?;
        records.push(record);
    }

    Ok(Relation::new_with_records(records))
}

fn write_relation_to_csv_file<M: Model>(
    relation: &Relation<M>,
    file_path: &PathBuf,
) -> Result<(), JoydbError> {
    let mut buffer = Vec::new();
    {
        let mut writer = Writer::from_writer(&mut buffer);
        for model in relation.records() {
            writer
                .serialize(model)
                .map_err(|e| JoydbError::Serialize(Box::new(e)))?;
        }

        writer.flush()?
    }

    fs_utils::safe_write(file_path, &buffer)?;

    Ok(())
}
