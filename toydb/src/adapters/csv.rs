use crate::adapters::{Adapter, Partitioned, PartitionedAdapter};
use crate::{Model, Relation};
use crate::{ToydbError, state::State};
use std::path::PathBuf;

pub struct CsvAdapter {
    dir_path: PathBuf,
}

impl CsvAdapter {
    pub fn new(dir_path: impl Into<PathBuf>) -> Self {
        Self {
            dir_path: dir_path.into(),
        }
    }

    fn relation_file_path<M: Model>(&self) -> PathBuf {
        self.dir_path.join(format!("{}.csv", M::relation_name()))
    }
}

impl PartitionedAdapter for CsvAdapter {
    fn write_relation<M: Model>(&self, relation: &Relation<M>) -> Result<(), ToydbError> {
        let file_path = self.relation_file_path::<M>();
        write_relation_to_csv_file(relation, &file_path)
    }

    fn load_relation<M: Model>(&self) -> Result<Relation<M>, ToydbError> {
        let file_path = self.relation_file_path::<M>();
        if file_path.exists() {
            if !file_path.is_file() {
                // If the path exists but is not a file, then return an error
                Err(ToydbError::NotFile(file_path))
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

    fn load_state<S: State>(&self) -> Result<S, ToydbError> {
        if self.dir_path.exists() {
            if !self.dir_path.is_dir() {
                return Err(ToydbError::NotDirectory(self.dir_path.clone()));
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

fn load_relation_from_csv_file<M: Model>(file_path: &PathBuf) -> Result<Relation<M>, ToydbError> {
    let mut reader =
        csv::Reader::from_path(file_path).map_err(|e| ToydbError::Deserialize(Box::new(e)))?;

    let mut models = Vec::new();
    for result in reader.deserialize() {
        let record: M = result.map_err(|e| ToydbError::Deserialize(Box::new(e)))?;
        models.push(record);
    }

    Ok(Relation::new_with_models(models))
}

fn write_relation_to_csv_file<M: Model>(
    relation: &Relation<M>,
    file_path: &PathBuf,
) -> Result<(), ToydbError> {
    let mut writer =
        csv::Writer::from_path(file_path).map_err(|e| ToydbError::Serialize(Box::new(e)))?;

    for model in relation.models() {
        writer
            .serialize(model)
            .map_err(|e| ToydbError::Serialize(Box::new(e)))?;
    }

    writer
        .flush()
        .map_err(|e| ToydbError::Serialize(Box::new(e)))?;
    Ok(())
}
