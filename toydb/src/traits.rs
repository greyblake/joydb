use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// An identifiable model that can be stored in a database.
pub trait Model: Clone + Serialize + Deserialize<'static> {
    type Id: Debug + Clone + Eq;

    fn id(&self) -> &Self::Id;
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
