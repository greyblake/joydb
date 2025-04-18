//! <p align="center"><img width="300" src="https://raw.githubusercontent.com/greyblake/joydb/master/art/rust_joydb_embedded_json_file_database.webp" alt="Joydb - a JSON/CSV database for Rust"></p>
//! <h2 align="center">JSON/CSV file database and ORM for quick prototyping.</h2>
//!
//! Joydb is a Rust library that acts like a database and ORM at the same time and provides a simple way to store and retrieve data in JSON, CSV or any other format.
//!
//! # Getting started
//! Install prerequisites:
//!
//! ```sh
//! cargo install serde --features derive
//! cargo install joydb --features json
//! ```
//!
//! ```
//! # let _ = ::std::fs::remove_file("data.json");
//!
//! use joydb::{Joydb, adapters::JsonAdapter, Model};
//! use serde::{Serialize, Deserialize};
//!
//! // Define your model
//! #[derive(Debug, Clone, Serialize, Deserialize, Model)]
//! struct User {
//!     // id is mandatory field for every model.
//!     // We use integer here, but most likely you will want to use Uuid.
//!     id: u32,
//!     username: String,
//! }
//!
//! // Define the state
//! joydb::state! {
//!     AppState,
//!     models: [User],
//! }
//!
//! // Define the database (combination of state and adapter)
//! type Db = Joydb<AppState, JsonAdapter>;
//!
//! // Create a new database or open an existing one
//! let db = Db::open("data.json").unwrap();
//!
//! let alice = User {
//!    id: 1,
//!    username: "Alice".to_string(),
//! };
//!
//! // Insert a new user
//! db.insert(&alice).unwrap();
//!
//! // Get the user by ID
//! let user = db.get::<User>(&1).unwrap().unwrap();
//! assert_eq!(user.username, "Alice");
//!
//! # let _ = ::std::fs::remove_file("data.json");
//! ```
//!
//! # CRUD operations
//!
//! | Operation | Methods                                                                                                      |
//! |-----------|--------------------------------------------------------------------------------------------------------------|
//! | Create    | [`insert`](Joydb::insert), [`upsert`](Joydb::upsert)                                                         |
//! | Read      | [`get`](Joydb::get), [`get_all`](Joydb::get_all), [`get_all_by`](Joydb::get_all_by), [`count`](Joydb::count) |
//! | Update    | [`update`](Joydb::update), [`upsert`](Joydb::upsert)                                                         |
//! | Delete    | [`delete`](Joydb::delete), [`delete_all_by`](Joydb::delete_all_by)                                           |
//!
//! Please refer to [Joydb] for more details.
//!
//! # Adapters
//!
//! There are 2 types of adapters:
//!
//! - _Unified_ - uses a single file to store the state. It writes and reads the entire state at once. Usually requires a file path.
//! - _Partitioned_ - uses multiple files to store the state. It writes and reads each relation separately. Usually requires directory path.
//!
//! The following adapters are implemented out of the box and can be used with the corresponding
//! feature flag enabled.
//!
//! | Adapter                                                           | Format | Type        | Feature flag |
//! | ----------------------------------------------------------------- | ------ | ----------- | ------------ |
//! | [JsonAdapter](crate::adapters::JsonAdapter)                       | JSON   | Unified     | `json`       |
//! | [JsonPartitionedAdapter](crate::adapters::JsonPartitionedAdapter) | JSON   | Partitioned | `json`       |
//! | [RonAdapter](crate::adapters::RonAdapter)                         | RON    | Unified     | `ron`        |
//! | [RonPartitionedAdapter](crate::adapters::RonPartitionedAdapter)   | RON    | Partitioned | `ron`        |
//! | [CsvAdapter](crate::adapters::CsvAdapter)                         | CSV    | Paritioned  | `csv`        |
//!
//! # Sync policy
//!
//! Sync policy defines when exactly the data must be written to the file system.
//!
//! Please see [SyncPolicy] for more details.
//!
//! # Motivation
//!
//! While prototyping new projects, I often needed some form of persistent storage.
//! However, setting up a full-fledged database and ORM felt like overkill for the project's scope.
//! So I'd occasionally fall back to a simple JSON file.
//! As this pattern repeated, I decided to solve the problem once and for all by building Joydb.
//!
//! # Limitation
//!
//! Joydb is designed in the way that it writes the entire database state to a file
//! system at once. This means that it is not suitable for high performance applications or for
//! domains where the data is too large to fit in memory.
//!
//! It's highly recommended to switch to a proper database like PostgreSQL before Joydb turns into
//! Paindb.
//!
//! # License
//!
//! MIT © [Serhii Potapov](https://www.greyblake.com)

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
