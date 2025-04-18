//! <p align="center"><img width="300" src="https://raw.githubusercontent.com/greyblake/joydb/master/art/rust_joydb_embedded_json_file_database.webp" alt="Joydb - a JSON/CSV database for Rust"></p>
//! <h2 align="center">JSON/CSV file database and ORM for quick prototyping.</h2>
//!
//! Joydb is a Rust library that acts like a database and ORM at the same time and provides a simple way to store and retrieve data in JSON, CSV or any other format.
//!
//! # Getting started
//!
//! ```
//! # let _ = ::std::fs::remove_file("data.json");
//!
//! use joydb::{Joydb, adapters::JsonAdapter, Model};
//! use serde::{Serialize, Deserialize};
//!
//! // Describe your model
//! #[derive(Debug, Clone, Serialize, Deserialize, Model)]
//! struct User {
//!     // id is mandatory field for every model
//!     id: u32,
//!     username: String,
//!     age: u32,
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
//!    age: 30,
//! };
//!
//! // Insert a new user
//! db.insert(&alice).unwrap();
//!
//! // Get the user by ID
//! let fetched_user = db.get::<User>(&1).unwrap().unwrap();
//! assert_eq!(fetched_user.username, "Alice");
//!
//! # let _ = ::std::fs::remove_file("data.json");
//! ```
//!
//! # CRUD operations
//!
//! [Joydb] provides all the regular CRUD operations.
//! Please refer to [Joydb] for more details.
//!
//! # Sync policy
//!
//! Sync policy defines when exactly data must be written to the file system.
//!
//! Please see [SyncPolicy] for more details.
//!
//! # Limitation
//!
//! Joydb is designed in the way that it writes the entire database state to a file
//! system at once. This means that it is not suitable for high performance applications or for
//! domains where the data is too large to fit in memory.
//!
//! It's highly recommended to switch to a proper database like PostgreSQL before Joydb turns into
//! Paindb.

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
