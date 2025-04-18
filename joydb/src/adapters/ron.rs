use crate::adapters::{
    Adapter, FromPath, Partitioned, PartitionedAdapter, Unified, UnifiedAdapter,
};
use crate::{JoydbError, state::State};
use crate::{Model, Relation};
use std::io::Read;
use std::path::{Path, PathBuf};

use super::fs_utils;

/// A RON adapter.
/// Stores the entire state in a single RON file.
///
/// For more information about RON (Rusty Object Notation), see [ron](https://docs.rs/ron/latest/ron/) crate.
pub struct RonAdapter {
    file_path: PathBuf,
    pretty_config: Option<ron::ser::PrettyConfig>,
}

impl FromPath for RonAdapter {
    fn from_path<P: AsRef<Path>>(file_path: P) -> Self {
        Self::new(file_path, true)
    }
}

impl RonAdapter {
    pub fn new<P: AsRef<Path>>(file_path: P, pretty: bool) -> Self {
        let pretty_config = if pretty {
            Some(ron::ser::PrettyConfig::default())
        } else {
            None
        };

        Self {
            file_path: file_path.as_ref().to_path_buf(),
            pretty_config,
        }
    }
}

impl UnifiedAdapter for RonAdapter {
    fn write_state<S: State>(&self, state: &S) -> Result<(), JoydbError> {
        write_to_file(state, &self.file_path, self.pretty_config.clone())
    }

    fn load_state<S: State>(&self) -> Result<S, JoydbError> {
        if self.file_path.exists() {
            if !self.file_path.is_file() {
                // If the path exists but is not a file, then return an error
                Err(JoydbError::NotFile(self.file_path.clone()))
            } else {
                // Otherwise read the state from the existing file
                let mut file = std::fs::File::open(&self.file_path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let state =
                    ron::from_str(&contents).map_err(|e| JoydbError::Deserialize(Box::new(e)))?;
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

impl Adapter for RonAdapter {
    type Target = Unified<Self>;
}

/// A RON adapter.
/// Stores every relation in a separate RON file.
///
/// For more information about RON (Rusty Object Notation), see [ron](https://docs.rs/ron/latest/ron/) crate.
pub struct RonPartitionedAdapter {
    dir_path: PathBuf,
    pretty_config: Option<ron::ser::PrettyConfig>,
}

impl FromPath for RonPartitionedAdapter {
    fn from_path<P: AsRef<Path>>(dir_path: P) -> Self {
        Self::new(dir_path, true)
    }
}

impl RonPartitionedAdapter {
    pub fn new<P: AsRef<Path>>(dir_path: P, pretty: bool) -> Self {
        let pretty_config = if pretty {
            Some(ron::ser::PrettyConfig::default())
        } else {
            None
        };
        Self {
            dir_path: dir_path.as_ref().to_path_buf(),
            pretty_config,
        }
    }

    fn relation_file_path<M: Model>(&self) -> PathBuf {
        self.dir_path.join(format!("{}.ron", M::model_name()))
    }
}

impl PartitionedAdapter for RonPartitionedAdapter {
    fn write_relation<M: Model>(&self, relation: &Relation<M>) -> Result<(), JoydbError> {
        write_to_file(
            relation,
            &self.relation_file_path::<M>(),
            self.pretty_config.clone(),
        )
    }

    fn load_relation<M: Model>(&self) -> Result<Relation<M>, JoydbError> {
        let file_path = self.relation_file_path::<M>();
        if file_path.exists() {
            if !file_path.is_file() {
                // If the path exists but is not a file, then return an error
                Err(JoydbError::NotFile(file_path))
            } else {
                // Otherwise read the relation from the existing file
                read_from_file(&file_path)
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

impl Adapter for RonPartitionedAdapter {
    type Target = Partitioned<Self>;
}

fn write_to_file<T: ::serde::Serialize>(
    data: &T,
    file_path: &PathBuf,
    pretty_config: Option<ron::ser::PrettyConfig>,
) -> Result<(), JoydbError> {
    let ron_string: String = if let Some(pretty_config) = pretty_config {
        ron::ser::to_string_pretty(data, pretty_config)
    } else {
        ron::ser::to_string(data)
    }
    .map_err(|e| JoydbError::Serialize(Box::new(e)))?;

    fs_utils::safe_write(file_path, ron_string.as_bytes())?;

    Ok(())
}

fn read_from_file<T: ::serde::de::DeserializeOwned>(file_path: &PathBuf) -> Result<T, JoydbError> {
    let content = fs_utils::read_file(file_path)?;
    let data = ron::de::from_str(&content).map_err(|e| JoydbError::Deserialize(Box::new(e)))?;
    Ok(data)
}
