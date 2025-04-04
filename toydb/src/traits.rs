use crate::State;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::Path;

use crate::ToydbError;

/// An identifiable model that can be stored in a database.
pub trait Model: Clone + Serialize + for<'de> Deserialize<'de> {
    type Id: Debug + Clone + Eq;

    fn id(&self) -> &Self::Id;

    fn relation_name() -> &'static str;
}

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
