// TODO: Put behind feature flag
mod csv;
mod json;

use crate::{JoydbError, state::State};
use crate::{Model, Relation};
pub use csv::CsvAdapter;
pub use json::{JsonAdapter, PartitionedJsonAdapter};
use std::marker::PhantomData;

// TODO:
// See: https://users.rust-lang.org/t/two-blanket-implementations-for-different-classes-of-objects/100173
// See example: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=db5ee78e4307b2ae4c1d113d0e39a6f2
//
// Introduce a main Adapter trait that can be implemented either through UnifiedAdapter or PartitionedAdapter.

// ------- ABSTRACTIONS --------- //

// TODO: Write a blog article about this workaround.
pub trait Adapter: Send + 'static {
    type Target: BlanketAdapter<Target = Self>;

    fn write_state<S: State>(&self, state: &S) -> Result<(), JoydbError> {
        Self::Target::write_state(self, state)
    }

    fn load_state<S: State>(&self) -> Result<S, JoydbError> {
        Self::Target::load_state(self)
    }
}

pub trait BlanketAdapter {
    type Target;
    fn write_state<S: State>(target: &Self::Target, state: &S) -> Result<(), JoydbError>;
    fn load_state<S: State>(target: &Self::Target) -> Result<S, JoydbError>;
}

// Imlpement Adapter though UnifiedAdapter
pub struct Unified<UA: UnifiedAdapter>(PhantomData<UA>);

impl<UA: UnifiedAdapter> BlanketAdapter for Unified<UA> {
    type Target = UA;

    fn write_state<S: State>(target: &UA, state: &S) -> Result<(), JoydbError> {
        target.write_state(state)
    }

    fn load_state<S: State>(target: &UA) -> Result<S, JoydbError> {
        target.load_state()
    }
}

// Implement Adapter though PartitionedAdapter
pub struct Partitioned<PA: PartitionedAdapter>(PhantomData<PA>);

impl<PA: PartitionedAdapter> BlanketAdapter for Partitioned<PA> {
    type Target = PA;

    fn write_state<S: State>(target: &PA, state: &S) -> Result<(), JoydbError> {
        S::write_with_partitioned_adapter(state, target)
    }

    fn load_state<S: State>(target: &PA) -> Result<S, JoydbError> {
        target.load_state()
    }
}

pub trait UnifiedAdapter {
    // fn read_state<S: State>(&self) -> Result<S, JoydbError>;
    fn write_state<S: State>(&self, state: &S) -> Result<(), JoydbError>;

    /// Is called only once when the database is opened or created.
    /// Usually the adapter should check if the files exist and if not, create them.
    fn load_state<S: State>(&self) -> Result<S, JoydbError>;
}

/// The idea behind this trait is to allow storing relations in separate files.
///
/// For example, if you have a `User` model and a `Post` model, you can store
/// `User` models in `users.json` and `Post` models in `posts.json`.
///
/// But at the moment it's postponed.
pub trait PartitionedAdapter {
    fn write_relation<M: Model>(&self, relation: &Relation<M>) -> Result<(), JoydbError>;

    fn load_state<S: State>(&self) -> Result<S, JoydbError>;

    // Is meant to be called by State, because State knows concrete type of M.
    fn load_relation<M: Model>(&self) -> Result<Relation<M>, JoydbError>;
}
