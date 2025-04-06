use std::num::ParseIntError;

use serde::{Deserialize, Serialize};
use toydb::{Backend, Model, PartitionedDb, PartitionedJsonAdapter};

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

toydb::define_state! {
    AppState,
    models: [User, Post],
}

type Db = PartitionedDb<AppState, PartitionedJsonAdapter>;

const DB_DIR: &str = "db_data";

fn main() {
    let backend = Backend::Partitioned(PartitionedJsonAdapter);
    let db = Db::open_with_backend(backend, DB_DIR).unwrap();

    db.insert(&User {
        id: 1,
        name: "Alice".to_owned(),
    })
    .unwrap();
    db.insert(&User {
        id: 2,
        name: "Bob".to_owned(),
    })
    .unwrap();

    db.insert(&Post {
        id: 1,
        title: "Hello, world!".to_owned(),
    })
    .unwrap();
}
