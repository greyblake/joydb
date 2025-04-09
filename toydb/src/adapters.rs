use crate::{Model, Relation};
use crate::{ToydbError, state::State};
use std::io::{Read, Write};
use std::path::PathBuf;

pub trait UnifiedAdapter {
    fn read_state<S: State>(&self) -> Result<S, ToydbError>;
    fn write_state<S: State>(&self, state: &S) -> Result<(), ToydbError>;

    /// Is called only once when the database is opened or created.
    /// Usually the adapter should check if the files exist and if not, create them.
    fn init_state<S: State>(&self) -> Result<S, ToydbError>;
}

// impl<UA: UnifiedAdapter> From<UA> for Backend<UA, NeverAdapter> {
//     fn from(adapter: UA) -> Self {
//         Self::Unified(adapter)
//     }
// }
//

/// The idea behind this trait is to allow storing relations in separate files.
///
/// For example, if you have a `User` model and a `Post` model, you can store
/// `User` models in `users.json` and `Post` models in `posts.json`.
///
/// But at the moment it's postponed.
pub trait PartitionedAdapter {
    fn read_relation<M: Model>(&self) -> Result<Relation<M>, ToydbError>;

    fn write_relation<M: Model>(&self, relation: &Relation<M>) -> Result<(), ToydbError>;

    fn init_state<S: State>(&self) -> Result<S, ToydbError>;

    // Is meant to be called by State, because State knows concrete type of M.
    fn init_relation<M: Model>(&self) -> Result<Relation<M>, ToydbError>;
}

// impl<PA: PartitionedAdapter> From<PA> for Backend<NeverAdapter, PA> {
//     fn from(adapter: PA) -> Self {
//         Self::Partitioned(adapter)
//     }
// }

// TODO: add `pretty` boolean?
pub struct UnifiedJsonAdapter {
    path: PathBuf,
}

impl UnifiedJsonAdapter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl UnifiedAdapter for UnifiedJsonAdapter {
    fn read_state<S: State>(&self) -> Result<S, ToydbError> {
        let mut file = std::fs::File::open(&self.path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let state = serde_json::from_str(&contents)?;
        Ok(state)
    }

    fn write_state<S: State>(&self, state: &S) -> Result<(), ToydbError> {
        let json = serde_json::to_string_pretty(state)?;
        let mut file = std::fs::File::create(&self.path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn init_state<S: State>(&self) -> Result<S, ToydbError> {
        if self.path.exists() {
            if !self.path.is_file() {
                // If the path exists but is not a file, then return an error
                return Err(ToydbError::NotFile(self.path.clone()));
            }
            // Otherwise read the state from the existing file
            self.read_state()
        } else {
            // If the file does not exist, create a new file with empty state
            let empty_state = S::default();
            self.write_state(&empty_state)?;
            Ok(empty_state)
        }
    }
}

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
    fn read_relation<M: Model>(&self) -> Result<Relation<M>, ToydbError> {
        let file_path = self.relation_file_path::<M>();
        let mut file = std::fs::File::open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let relation = serde_json::from_str(&contents)?;
        Ok(relation)
    }

    fn write_relation<M: Model>(&self, relation: &Relation<M>) -> Result<(), ToydbError> {
        let file_path = self.relation_file_path::<M>();
        let json = serde_json::to_string_pretty(relation)?;
        let mut file = std::fs::File::create(&file_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn init_relation<M: Model>(&self) -> Result<Relation<M>, ToydbError> {
        let file_path = self.relation_file_path::<M>();
        if file_path.exists() {
            if !file_path.is_file() {
                // If the path exists but is not a file, then return an error
                return Err(ToydbError::NotFile(file_path));
            }
            // Otherwise read the relation from the existing file
            self.read_relation()
        } else {
            // If the file does not exist, create a new file with empty relation
            let empty_relation = Relation::<M>::default();
            self.write_relation(&empty_relation)?;
            Ok(empty_relation)
        }
    }

    fn init_state<S: State>(&self) -> Result<S, ToydbError> {
        if self.dir_path.exists() {
            if !self.dir_path.is_dir() {
                return Err(ToydbError::NotDirectory(self.dir_path.clone()));
            }
        } else {
            // Create a directory if it does not exist
            std::fs::create_dir_all(&self.dir_path)?;
        }

        S::init_with_partitioned_adapter(self)
    }
}

#[derive(Debug)]
pub struct NeverAdapter;

impl UnifiedAdapter for NeverAdapter {
    fn read_state<S: State>(&self) -> Result<S, ToydbError> {
        panic!("NeverAdapter is not meant to be used as UnifiedAdapter to read.");
    }

    fn write_state<S: State>(&self, _state: &S) -> Result<(), ToydbError> {
        panic!("NeverAdapter is not meant to be used as UnifiedAdapter to write.");
    }

    fn init_state<S: State>(&self) -> Result<S, ToydbError> {
        panic!("NeverAdapter is not meant to be used as UnifiedAdapter to init.");
    }
}

impl PartitionedAdapter for NeverAdapter {
    fn read_relation<M: Model>(&self) -> Result<Relation<M>, ToydbError> {
        panic!("NeverAdapter is not meant to be used as PartitionedAdapter to read.");
    }

    fn write_relation<M: Model>(&self, _relation: &Relation<M>) -> Result<(), ToydbError> {
        panic!("NeverAdapter is not meant to be used as PartitionedAdapter to write.");
    }

    fn init_state<S: State>(&self) -> Result<S, ToydbError> {
        panic!("NeverAdapter is not meant to be used as PartitionedAdapter to init_state.");
    }

    fn init_relation<M: Model>(&self) -> Result<Relation<M>, ToydbError> {
        panic!("NeverAdapter is not meant to be used as PartitionedAdapter to init relation.");
    }
}

#[derive(Debug)]
pub enum Backend<UA: UnifiedAdapter, PA: PartitionedAdapter> {
    Unified(UA),
    Partitioned(PA),
}
