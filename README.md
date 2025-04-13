# Joydb

An in-memory embedded database with persistence and multiple adapters (JSON, CSV, etc).
Acts like a minimalistic ORM with zero setup.
Simple, lightweight, and perfect for prototypes, small apps, or experiments.

## Get started

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
joydb::define_state! {
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
