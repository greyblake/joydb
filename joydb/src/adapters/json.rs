use crate::adapters::{
    Adapter, FromPath, Partitioned, PartitionedAdapter, Unified, UnifiedAdapter,
};
use crate::{JoydbError, state::State};
use crate::{Model, Relation};
use std::path::{Path, PathBuf};

use super::fs_utils;

pub struct JsonAdapter {
    /// Path to the JSON file where the state is stored.
    file_path: PathBuf,

    /// Whether to pretty-print the JSON output. By default is `true`.
    pretty: bool,
}

impl FromPath for JsonAdapter {
    fn from_path<P: AsRef<Path>>(file_path: P) -> Self {
        Self::new(file_path, true)
    }
}

impl JsonAdapter {
    pub fn new<P: AsRef<Path>>(file_path: P, pretty: bool) -> Self {
        Self {
            file_path: file_path.as_ref().to_path_buf(),
            pretty,
        }
    }
}

impl UnifiedAdapter for JsonAdapter {
    fn write_state<S: State>(&self, state: &S) -> Result<(), JoydbError> {
        write_to_file(state, &self.file_path, self.pretty)
    }

    fn load_state<S: State>(&self) -> Result<S, JoydbError> {
        if self.file_path.exists() {
            if !self.file_path.is_file() {
                // If the path exists but is not a file, then return an error
                Err(JoydbError::NotFile(self.file_path.clone()))
            } else {
                // Otherwise read the state from the existing file
                read_from_file::<S>(&self.file_path)
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

pub struct JsonPartitionedAdapter {
    /// Path to the directory where the partitioned JSON files are stored.
    dir_path: PathBuf,

    /// Whether to pretty-print the JSON output. By default is `true`.
    pretty: bool,
}

impl FromPath for JsonPartitionedAdapter {
    fn from_path<P: AsRef<Path>>(dir_path: P) -> Self {
        Self::new(dir_path, true)
    }
}

impl JsonPartitionedAdapter {
    pub fn new<P: AsRef<Path>>(dir_path: P, pretty: bool) -> Self {
        Self {
            dir_path: dir_path.as_ref().to_path_buf(),
            pretty,
        }
    }

    /// Build the file path for the relation of a given model.
    fn relation_file_path<M: Model>(&self) -> PathBuf {
        self.dir_path.join(format!("{}.json", M::relation_name()))
    }
}

impl PartitionedAdapter for JsonPartitionedAdapter {
    fn write_relation<M: Model>(&self, relation: &Relation<M>) -> Result<(), JoydbError> {
        write_to_file(relation, &self.relation_file_path::<M>(), self.pretty)
    }

    fn load_relation<M: Model>(&self) -> Result<Relation<M>, JoydbError> {
        let file_path = self.relation_file_path::<M>();
        if file_path.exists() {
            if !file_path.is_file() {
                // If the path exists but is not a file, then return an error
                Err(JoydbError::NotFile(file_path))
            } else {
                read_from_file::<Relation<M>>(&file_path)
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

impl Adapter for JsonPartitionedAdapter {
    type Target = Partitioned<Self>;
}

fn write_to_file<T: ::serde::Serialize>(
    data: &T,
    file_path: &PathBuf,
    pretty: bool,
) -> Result<(), JoydbError> {
    let json_string = if pretty {
        serde_json::to_string_pretty(data)
    } else {
        serde_json::to_string(data)
    }
    .map_err(|e| JoydbError::Serialize(Box::new(e)))?;

    fs_utils::safe_write(file_path, json_string.as_bytes())?;

    Ok(())
}

fn read_from_file<T: ::serde::de::DeserializeOwned>(file_path: &PathBuf) -> Result<T, JoydbError> {
    let content = fs_utils::read_file(file_path)?;
    let data = serde_json::from_str(&content).map_err(|e| JoydbError::Deserialize(Box::new(e)))?;
    Ok(data)
}
