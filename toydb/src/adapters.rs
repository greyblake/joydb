use crate::{Model, Relation};
use crate::{ToydbError, state::State};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub trait UnifiedAdapter {
    fn read<S: State>(&self) -> Result<S, ToydbError>;
    fn write<S: State>(&self, state: &S) -> Result<(), ToydbError>;
}

/// The idea behind this trait is to allow storing relations in separate files.
///
/// For example, if you have a `User` model and a `Post` model, you can store
/// `User` models in `users.json` and `Post` models in `posts.json`.
///
/// But at the moment it's postponed.
pub trait RelationAdapter {
    fn read<M: Model>(base_path: &Path) -> Result<Relation<M>, ToydbError>;
    fn write<M: Model>(base_path: &Path, relation: &Relation<M>) -> Result<(), ToydbError>;
}

pub struct UnifiedJsonAdapter {
    path: PathBuf,
}

impl UnifiedJsonAdapter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl UnifiedAdapter for UnifiedJsonAdapter {
    fn read<S: State>(&self) -> Result<S, ToydbError> {
        let mut file = std::fs::File::open(&self.path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let state = serde_json::from_str(&contents)?;
        Ok(state)
    }

    fn write<S: State>(&self, state: &S) -> Result<(), ToydbError> {
        let json = serde_json::to_string_pretty(state)?;
        let mut file = std::fs::File::create(&self.path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

pub struct PartitionedJsonAdapter;

impl RelationAdapter for PartitionedJsonAdapter {
    fn read<M: Model>(base_path: &Path) -> Result<Relation<M>, ToydbError> {
        let path = base_path.join(format!("{}.json", M::relation_name()));
        let mut file = std::fs::File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let relation = serde_json::from_str(&contents)?;
        Ok(relation)
    }

    fn write<M: Model>(base_path: &Path, relation: &Relation<M>) -> Result<(), ToydbError> {
        let path = base_path.join(format!("{}.json", M::relation_name()));
        let json = serde_json::to_string_pretty(relation)?;
        let mut file = std::fs::File::create(&path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct NeverAdapter;

impl UnifiedAdapter for NeverAdapter {
    fn read<S: State>(&self) -> Result<S, ToydbError> {
        panic!("NeverAdapter is not meant to be used as UnifiedAdapter to read.");
    }

    fn write<S: State>(&self, _state: &S) -> Result<(), ToydbError> {
        panic!("NeverAdapter is not meant to be used as UnifiedAdapter to write.");
    }
}

impl RelationAdapter for NeverAdapter {
    fn read<M: Model>(_path: &Path) -> Result<Relation<M>, ToydbError> {
        panic!("NeverAdapter is not meant to be used as RelationAdapter to read.");
    }

    fn write<M: Model>(_path: &Path, _relation: &Relation<M>) -> Result<(), ToydbError> {
        panic!("NeverAdapter is not meant to be used as RelationAdapter to write.");
    }
}

#[derive(Debug)]
pub enum Backend<UA: UnifiedAdapter, RA: RelationAdapter> {
    Unified(UA),
    Partitioned(RA),
}
