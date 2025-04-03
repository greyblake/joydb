use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::path::Path;

use crate::Relation;

/// An identifiable model that can be stored in a database.
pub trait Model: Clone + Serialize + for<'de> Deserialize<'de> {
    type Id: Debug + Clone + Eq;

    fn id(&self) -> &Self::Id;

    fn relation_name() -> &'static str;
}

/// A utility trait that implemented by a state that can store a relation of a model.
#[diagnostic::on_unimplemented(
    message = "State `{Self}` does not doest not implement `GetRelation<{M}>`.\nDid you forget to add `{M}` in the state definition?",
    note = "Make sure that model `{M}` is listed in the state definition."
)]
pub trait GetRelation<M: Model> {
    fn get_rel_mut(&mut self) -> &mut Relation<M>;

    fn get_rel(&self) -> &Relation<M>;
}

pub trait State: Default + Debug + Serialize + DeserializeOwned {
    fn is_dirty(&self) -> bool;

    fn reset_dirty(&mut self);
}

pub trait Adapter {
    fn read<S: State>(path: &Path) -> S;
    fn write<S: State>(path: &Path, state: &S);
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
