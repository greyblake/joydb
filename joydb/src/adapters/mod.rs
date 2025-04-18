//! Common place for adapter abstractions and a few implementations.
//!
//! ## Unified VS Partitioned adapters
//!
//! There are 2 types of adapters:
//!
//! - _Unified_ - uses a single file to store the state. It writes and reads the entire state at once. Usually requires a file path.
//! - _Partitioned_ - uses multiple files to store the state. It writes and reads each relation separately. Usually requires directory path.
//!
//! ## Supported adapters
//!
//! The following adapters are implemented out of the box and can be used with the corresponding
//! feature flag enabled.
//!
//! | Adapter                  | Format | Type        | Feature flag |
//! | ------------------------ | ------ | ----------- | ------------ |
//! | [JsonAdapter]            | JSON   | Unified     | `json`       |
//! | [JsonPartitionedAdapter] | JSON   | Partitioned | `json`       |
//! | [RonAdapter]             | RON    | Unified     | `ron`        |
//! | [RonPartitionedAdapter]  | RON    | Partitioned | `ron`        |
//! | [CsvAdapter]             | CSV    | Paritioned  | `csv`        |
//!
//!

#[cfg(feature = "csv")]
mod csv;

#[cfg(feature = "csv")]
pub use csv::CsvAdapter;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "json")]
pub use json::{JsonAdapter, JsonPartitionedAdapter};

#[cfg(feature = "ron")]
mod ron;

#[cfg(feature = "ron")]
pub use ron::{RonAdapter, RonPartitionedAdapter};

use crate::{JoydbError, state::State};
use crate::{Model, Relation};
use std::marker::PhantomData;
use std::path::Path;

mod fs_utils;

// ------- ABSTRACTIONS --------- //

// TODO: Write a blog article about this workaround.
// See: https://users.rust-lang.org/t/two-blanket-implementations-for-different-classes-of-objects/100173
// See example: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=db5ee78e4307b2ae4c1d113d0e39a6f2

/// A trait that every adapter must implement.
/// Adapter determines how to write and how to load the state from the file system
/// (or any other storage).
///
/// A concrete adapter must be implemented  either though [UnifiedAdapter] or [PartitionedAdapter].
pub trait Adapter: Send + 'static {
    type Target: BlanketAdapter<Target = Self>;

    /// Write the state to the file system or any other storage.
    fn write_state<S: State>(&self, state: &S) -> Result<(), JoydbError> {
        Self::Target::write_state(self, state)
    }

    /// Load the state from the file system or any other storage.
    fn load_state<S: State>(&self) -> Result<S, JoydbError> {
        Self::Target::load_state(self)
    }
}

/// A tiny helper trait that allows to implement [Adapter] trait in terms of
/// either [UnifiedAdapter] or [PartitionedAdapter] traits.
pub trait BlanketAdapter {
    type Target;
    fn write_state<S: State>(target: &Self::Target, state: &S) -> Result<(), JoydbError>;
    fn load_state<S: State>(target: &Self::Target) -> Result<S, JoydbError>;
}

/// A utility struct that implements [BlanketAdapter] trait though in terms of [UnifiedAdapter].
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

/// A utility struct that implements [BlanketAdapter] trait though in terms of [PartitionedAdapter].
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

/// The trait is used to define the adapters that use a single file to store the state.
pub trait UnifiedAdapter {
    /// Write the state to the file.
    /// It's called every time when the database flushes a dirty state to the disk
    /// with [`Joydb::flush`](crate::Joydb::flush) method.
    fn write_state<S: State>(&self, state: &S) -> Result<(), JoydbError>;

    /// Is called only once when the database is opened or created.
    /// Usually the adapter should check if the files exist and if not, create them.
    fn load_state<S: State>(&self) -> Result<S, JoydbError>;
}

/// The idea behind this trait is to allow storing relations in separate files.
///
/// For example, if you have a `User` model and a `Post` model, you can store
/// `User` models in `User.json` and `Post` models in `Post.json`.
///
/// But at the moment it's postponed.
pub trait PartitionedAdapter {
    /// Write a relation to a file system or any other storage.
    fn write_relation<M: Model>(&self, relation: &Relation<M>) -> Result<(), JoydbError>;

    /// Load the entire state (all relations) using the given partitioned adapter.
    fn load_state<S: State>(&self) -> Result<S, JoydbError>;

    /// Load a relation from a file system or any other storage.
    ///
    /// It's meant to be called by implementation of [crate::State], because State knows concrete type of M.
    fn load_relation<M: Model>(&self) -> Result<Relation<M>, JoydbError>;
}

/// This trait is used to create an adapter from a path.
/// Most of the adapters are file system based, so this enables better ergonomics
/// like opening a database with `Db::open("db.json")` instead of building the entire adapter
/// and config manually.
pub trait FromPath {
    fn from_path<P: AsRef<Path>>(path: P) -> Self;
}
