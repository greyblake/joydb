use crate::adapters::{Adapter, Partitioned, PartitionedAdapter, Unified, UnifiedAdapter};
use crate::{Model, Relation};
use crate::{JoydbError, state::State};
use std::io::{Read, Write};
use std::path::PathBuf;

// TODO: add `pretty` boolean?
pub struct JsonAdapter {
    path: PathBuf,
}

impl JsonAdapter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl UnifiedAdapter for JsonAdapter {
    fn write_state<S: State>(&self, state: &S) -> Result<(), JoydbError> {
        let json =
            serde_json::to_string_pretty(state).map_err(|e| JoydbError::Serialize(Box::new(e)))?;
        let mut file = std::fs::File::create(&self.path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn load_state<S: State>(&self) -> Result<S, JoydbError> {
        if self.path.exists() {
            if !self.path.is_file() {
                // If the path exists but is not a file, then return an error
                Err(JoydbError::NotFile(self.path.clone()))
            } else {
                // Otherwise read the state from the existing file
                let mut file = std::fs::File::open(&self.path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let state = serde_json::from_str(&contents)
                    .map_err(|e| JoydbError::Deserialize(Box::new(e)))?;
                Ok(state)
            }
        } else {
            // If the file does not exist, create a new file with empty state
            let empty_state = S::default();
            UnifiedAdapter::write_state(self, &empty_state)?;
            Ok(empty_state)
        }
    }
}

impl Adapter for JsonAdapter {
    type Target = Unified<Self>;
}

// --------------------------- //

pub struct PartitionedJsonAdapter {
    dir_path: PathBuf,
}

impl PartitionedJsonAdapter {
    pub fn new(dir_path: impl Into<PathBuf>) -> Self {
        Self {
            dir_path: dir_path.into(),
        }
    }

    fn relation_file_path<M: Model>(&self) -> PathBuf {
        self.dir_path.join(format!("{}.json", M::relation_name()))
    }
}

impl PartitionedAdapter for PartitionedJsonAdapter {
    fn write_relation<M: Model>(&self, relation: &Relation<M>) -> Result<(), JoydbError> {
        let file_path = self.relation_file_path::<M>();
        let json = serde_json::to_string_pretty(relation)
            .map_err(|e| JoydbError::Serialize(Box::new(e)))?;
        let mut file = std::fs::File::create(&file_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
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
                let mut file = std::fs::File::open(&file_path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let relation = serde_json::from_str(&contents)
                    .map_err(|e| JoydbError::Deserialize(Box::new(e)))?;
                Ok(relation)
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

impl Adapter for PartitionedJsonAdapter {
    type Target = Partitioned<Self>;
}
