use crate::{ToydbError, state::State};
use std::io::{Read, Write};
use std::path::Path;


pub trait Adapter {
    fn read<S: State>(path: &Path) -> Result<S, ToydbError>;
    fn write<S: State>(path: &Path, state: &S) -> Result<(), ToydbError>;
}

/*
/// The idea behind this trait is to allow storing relations in separate files.
///
/// For example, if you have a `User` model and a `Post` model, you can store
/// `User` models in `users.json` and `Post` models in `posts.json`.
///
/// But at the moment it's postponed.
pub trait RelationAdapter<M: Model> {
    const EXTENSION: &'static str;

    // TODO: Make it return Result
    fn deserialize(&self, file_content: Vec<u8>) -> Vec<M>;

    // TODO: Make it return Result
    fn serialize(&self, models: Vec<M>) -> Vec<u8>;
}
*/

pub struct JsonAdapter;

impl Adapter for JsonAdapter {
    fn read<S: State>(path: &Path) -> Result<S, ToydbError> {
        let mut file = std::fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let state = serde_json::from_str(&contents)?;
        Ok(state)
    }

    fn write<S: State>(path: &Path, state: &S) -> Result<(), ToydbError> {
        let json = serde_json::to_string_pretty(state)?;
        let mut file = std::fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
