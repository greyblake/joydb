// TODO: Put behind feature flag
mod json;

use crate::{Model, Relation};
use crate::{ToydbError, state::State};
pub use json::{JsonAdapter, PartitionedJsonAdapter};
use std::marker::PhantomData;

// TODO:
// See: https://users.rust-lang.org/t/two-blanket-implementations-for-different-classes-of-objects/100173
// See example: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=db5ee78e4307b2ae4c1d113d0e39a6f2
//
// Introduce a main Adapter trait that can be implemented either through UnifiedAdapter or PartitionedAdapter.

// ------- ABSTRACTIONS --------- //

// TODO: Write a blog article about this workaround.
pub trait Adapter {
    type Target: BlanketAdapter<Target = Self>;

    fn write_state<S: State>(&self, state: &S) -> Result<(), ToydbError> {
        Self::Target::write_state(self, state)
    }

    fn init_state<S: State>(&self) -> Result<S, ToydbError> {
        Self::Target::init_state(self)
    }
}

pub trait BlanketAdapter {
    type Target;
    fn write_state<S: State>(target: &Self::Target, state: &S) -> Result<(), ToydbError>;
    fn init_state<S: State>(target: &Self::Target) -> Result<S, ToydbError>;
}

// Imlpement Adapter though UnifiedAdapter
pub struct Unified<UA: UnifiedAdapter>(PhantomData<UA>);

impl<UA: UnifiedAdapter> BlanketAdapter for Unified<UA> {
    type Target = UA;

    fn write_state<S: State>(target: &UA, state: &S) -> Result<(), ToydbError> {
        target.write_state(state)
    }

    fn init_state<S: State>(target: &UA) -> Result<S, ToydbError> {
        target.init_state()
    }
}

// Implement Adapter though PartitionedAdapter
pub struct Partitioned<PA: PartitionedAdapter>(PhantomData<PA>);

impl<PA: PartitionedAdapter> BlanketAdapter for Partitioned<PA> {
    type Target = PA;

    fn write_state<S: State>(target: &PA, state: &S) -> Result<(), ToydbError> {
        S::write_with_partitioned_adapter(state, target)
    }

    fn init_state<S: State>(target: &PA) -> Result<S, ToydbError> {
        target.init_state()
    }
}

pub trait UnifiedAdapter {
    // fn read_state<S: State>(&self) -> Result<S, ToydbError>;
    fn write_state<S: State>(&self, state: &S) -> Result<(), ToydbError>;

    /// Is called only once when the database is opened or created.
    /// Usually the adapter should check if the files exist and if not, create them.
    fn init_state<S: State>(&self) -> Result<S, ToydbError>;
}

/// The idea behind this trait is to allow storing relations in separate files.
///
/// For example, if you have a `User` model and a `Post` model, you can store
/// `User` models in `users.json` and `Post` models in `posts.json`.
///
/// But at the moment it's postponed.
pub trait PartitionedAdapter {
    fn write_relation<M: Model>(&self, relation: &Relation<M>) -> Result<(), ToydbError>;

    fn init_state<S: State>(&self) -> Result<S, ToydbError>;

    // Is meant to be called by State, because State knows concrete type of M.
    fn init_relation<M: Model>(&self) -> Result<Relation<M>, ToydbError>;
}
