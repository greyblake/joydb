use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// An abstraction that allows to identify an entity by its id.
pub trait Model: Clone + Serialize + Deserialize<'static> {
    type Id: Debug + Clone + Copy + Eq;

    fn id(&self) -> Self::Id;
}
