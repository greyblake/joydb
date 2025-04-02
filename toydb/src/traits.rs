use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::path::Path;

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
    fn get_rel_mut(&mut self) -> &mut Vec<M>;

    fn get_rel(&self) -> &Vec<M>;
}

pub trait State {
    /// A state with data only, without any metal information.
    type PlainState: Serialize + DeserializeOwned;

    fn to_plain(&self) -> Self::PlainState;

    fn from_plain(plain: Self::PlainState) -> Self;
}

pub trait Adapter<S: State> {
    fn read(&self, path: &Path) -> S;
    fn write(&self, path: &Path, state: S);
}

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
