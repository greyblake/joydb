use crate::{Model, Relation};
use crate::{ToydbError, state::State};
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::PathBuf;

// TODO:
// See: https://users.rust-lang.org/t/two-blanket-implementations-for-different-classes-of-objects/100173
// See example: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=db5ee78e4307b2ae4c1d113d0e39a6f2
//
// Introduce a main Adapter trait that can be implemented either through UnifiedAdapter or PartitionedAdapter.

// ------- ABSTRACTIONS --------- //

// TODO: Write a blog article about this workaround.
pub trait Adapter {
    type Target: BlanketAdapter<Target = Self>;

    fn read_state<S: State>(&self) -> Result<S, ToydbError> {
        Self::Target::read_state(self)
    }

    fn write_state<S: State>(&self, state: &S) -> Result<(), ToydbError> {
        Self::Target::write_state(self, state)
    }

    fn init_state<S: State>(&self) -> Result<S, ToydbError> {
        Self::Target::init_state(self)
    }
}

pub trait BlanketAdapter {
    type Target;
    fn read_state<S: State>(target: &Self::Target) -> Result<S, ToydbError>;
    fn write_state<S: State>(target: &Self::Target, state: &S) -> Result<(), ToydbError>;
    fn init_state<S: State>(target: &Self::Target) -> Result<S, ToydbError>;
}

// Imlpement Adapter though UnifiedAdapter
pub struct Unified<UA: UnifiedAdapter>(PhantomData<UA>);

impl<UA: UnifiedAdapter> BlanketAdapter for Unified<UA> {
    type Target = UA;

    fn read_state<S: State>(target: &UA) -> Result<S, ToydbError> {
        target.read_state()
    }

    fn write_state<S: State>(target: &UA, state: &S) -> Result<(), ToydbError> {
        target.write_state(state)
    }

    fn init_state<S: State>(target: &UA) -> Result<S, ToydbError> {
        target.init_state()
    }
}

// Implement Adapter though PartitionedAdapter
pub struct Partitioned<PA: PartitionedAdapter>(PhantomData<PA>);

impl<PA: PartitionedAdapter> BlanketAdapter for Partitioned<PA> {
    type Target = PA;

    fn read_state<S: State>(target: &PA) -> Result<S, ToydbError> {
        S::load_with_partitioned_adapter(target)
    }

    fn write_state<S: State>(target: &PA, state: &S) -> Result<(), ToydbError> {
        S::write_with_partitioned_adapter(state, target)
    }

    fn init_state<S: State>(target: &PA) -> Result<S, ToydbError> {
        target.init_state()
    }
}

pub trait UnifiedAdapter {
    fn read_state<S: State>(&self) -> Result<S, ToydbError>;
    fn write_state<S: State>(&self, state: &S) -> Result<(), ToydbError>;

    /// Is called only once when the database is opened or created.
    /// Usually the adapter should check if the files exist and if not, create them.
    fn init_state<S: State>(&self) -> Result<S, ToydbError>;
}

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
            UnifiedAdapter::read_state(self)
        } else {
            // If the file does not exist, create a new file with empty state
            let empty_state = S::default();
            UnifiedAdapter::write_state(self, &empty_state)?;
            Ok(empty_state)
        }
    }
}

impl Adapter for UnifiedJsonAdapter {
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

impl Adapter for PartitionedJsonAdapter {
    type Target = Partitioned<Self>;
}
