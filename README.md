<p align="center">
<picture>
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/greyblake/joydb/master/art/rust_joydb_embedded_json_file_database.webp">
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/greyblake/joydb/master/art/rust_joydb_embedded_json_file_database_inverted.webp">

  <img width="300" alt="Rust Joydb Logo (Rust JSON embedded database)" src="https://raw.githubusercontent.com/greyblake/joydb/master/art/rust_joydb_embedded_json_file_database.webp">
</picture>
</p>
<h2 align="center">JSON file database and ORM for quick prototyping.</h2>

An in-memory embedded database with persistence and multiple adapters (JSON, CSV, etc).
Acts like a minimalistic ORM with zero setup.
Simple, lightweight, and perfect for prototypes, small apps, or experiments.
Not intended for serious production use, optimized for nothing but ergonomics.

## Get started

Install prerequisites:

```
cargo install serde --features derive
cargo install joydb --features json
```

Example:

```rust
use joydb::{Joydb, Model, adapters::JsonAdapter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
struct User {
    id: u32,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
struct Post {
    id: u32,
    title: String,
}

// Define the state by listing the models
joydb::state! {
    AppState,
    models: [User, Post],
}

// Define your the database.
// Typewise it's essentially combination of the state and adapter.
type Db = Joydb<AppState, JsonAdapter>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Db::open("data.json")?;

    // Insert a new record
    db.insert(&User {
        id: 1,
        name: "Alice".to_owned(),
    })?;

    // Get a record by ID
    let alice = db.find::<User>(&1)?.unwrap();
    assert_eq!(alice.name, "Alice");
}
```



## Similar projects

- [lowdb](https://github.com/typicode/lowdb) - JSON database for JavaScript
- [alkali](https://github.com/kneufeld/alkali) - Python ORM that writes to disk (JSON, YAML, CSV, etc)
