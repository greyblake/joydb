use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// An identifiable model that can be stored in a database.
pub trait Model: Clone + Serialize + for<'de> Deserialize<'de> {
    type Id: Debug + Clone + Eq;

    fn id(&self) -> &Self::Id;

    fn relation_name() -> &'static str;
}
