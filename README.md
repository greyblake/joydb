<p align="center">
<picture>
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/greyblake/joydb/master/art/rust_joydb_embedded_json_file_database.webp">
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/greyblake/joydb/master/art/rust_joydb_embedded_json_file_database_inverted.webp">

  <img width="300" alt="Rust Joydb Logo (Rust JSON embedded database)" src="https://raw.githubusercontent.com/greyblake/joydb/master/art/rust_joydb_embedded_json_file_database.webp">
</picture>
</p>
<h2 align="center">JSON/CSV file database and ORM for quick prototyping.</h2>


<p align="center">
<a href="https://github.com/greyblake/joydb/actions/workflows/ci.yml" rel="nofollow"><img src="https://github.com/greyblake/joydb/actions/workflows/ci.yml/badge.svg" alt="Nutype Build Status"></a>
<a href="https://docs.rs/joydb" rel="nofollow"><img src="https://docs.rs/joydb/badge.svg" alt="Joydb Documentation"></a>
<p>

An in-memory embedded database with persistence and multiple adapters.
Acts like a minimalistic ORM.
Good for prototypes and quick experiments.
Not intended for serious production use. Optimized for nothing but ergonomics.

## Get started

Install prerequisites:

```
cargo install serde --features derive
cargo install joydb --features json
```

Example:

```rust
use joydb::{Joydb, adapters::JsonAdapter, Model};
use serde::{Serialize, Deserialize};

// Define your model
#[derive(Debug, Clone, Serialize, Deserialize, Model)]
struct User {
    // id is mandatory field for every model.
    // We use integer here, but most likely you will want to use Uuid.
    id: u32,
    username: String,
}

// Define the state
joydb::state! {
    AppState,
    models: [User],
}

// Define the database (combination of state and adapter)
type Db = Joydb<AppState, JsonAdapter>;

// Create a new database or open an existing one
let db = Db::open("./data.json").unwrap();

let alice = User {
   id: 1,
   username: "Alice".to_string(),
};

// Insert a new user
db.insert(&alice).unwrap();

// Get the user by ID
let user = db.get::<User>(&1).unwrap().unwrap();
assert_eq!(user.username, "Alice");
```
## CRUD operations

| Operation | Methods                                 |
|-----------|-----------------------------------------|
| Create    | `insert`, `upsert`                      |
| Read      | `get`, `get_all`, `get_all_by`, `count` |
| Update    | `update`, `upsert`                      |
| Delete    | `delete`, `delete_all_by`               |

Please refer to [Joydb](https://docs.rs/joydb/latest/joydb/struct.Joydb.html#crud-operations) for more details.

## Adapters

There are 2 types of adapters:

- _Unified_ - uses a single file to store the state. It writes and reads the entire state at once. Usually requires a file path.
- _Partitioned_ - uses multiple files to store the state. It writes and reads each relation separately. Usually requires directory path.

The following adapters are implemented out of the box and can be used with the corresponding
feature flag enabled.

| Adapter                   | Format | Type        | Feature flag |
|---------------------------|--------|-------------|--------------|
| `JsonAdapter`             | JSON   | Unified     | `json`       |
| `JsonPartitionedAdapter`  | JSON   | Partitioned | `json`       |
| `RonAdapter`              | RON    | Unified     | `ron`        |
| `RonPartitionedAdapter`   | RON    | Partitioned | `ron`        |
| `CsvAdapter`              | CSV    | Paritioned  | `csv`        |

## Sync policy

Sync policy defines when exactly the data must be written to the file system.

Please see [SyncPolicy](https://docs.rs/joydb/latest/joydb/enum.SyncPolicy.html) for more details.


## Motivation

While prototyping new projects, I often needed some form of persistent storage.
However, setting up a full-fledged database and ORM felt like overkill for the project's scope.
So I'd occasionally fall back to a simple JSON file.
As this pattern repeated, I decided to solve the problem once and for all by building Joydb.


## Limitations

Joydb is designed in the way that it writes the entire database state to a file
system at once. This means that it is not suitable for high performance applications or for
domains where the data is too large to fit in memory.

It's highly recommended to switch to a proper database like PostgreSQL before Joydb turns into
Paindb.


## Similar projects

- [lowdb](https://github.com/typicode/lowdb) - JSON database for JavaScript
- [alkali](https://github.com/kneufeld/alkali) - Python ORM that writes to disk (JSON, YAML, CSV, etc)


## License

MIT © [Serhii Potapov](https://www.greyblake.com)
