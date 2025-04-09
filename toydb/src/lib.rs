mod adapters;
mod db;
mod error;
mod model;
mod relation;
mod state;

pub use adapters::{
    Backend, NeverAdapter, PartitionedAdapter, PartitionedJsonAdapter, UnifiedAdapter,
    UnifiedJsonAdapter,
};
pub use db::{PartitionedDb, Toydb, UnifiedDb};
pub use error::ToydbError;
pub use model::Model;
pub use relation::Relation;
pub use state::{GetRelation, State};

pub use toydb_macros::Model;
