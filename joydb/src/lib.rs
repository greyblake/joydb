pub mod adapters;
mod db;
mod error;
mod model;
mod relation;
mod state;

pub use db::{Joydb, JoydbConfig, JoydbMode, SyncPolicy};
pub use error::JoydbError;
pub use model::Model;
pub use relation::Relation;
pub use state::{GetRelation, State};

/// A macro to derive the [Model] trait for a struct.
/// A struct must have a field named `id`, which is the primary key.
pub use joydb_macros::Model;
