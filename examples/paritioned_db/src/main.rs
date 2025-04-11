use serde::{Deserialize, Serialize};
use toydb::{Model, Toydb, adapters::PartitionedJsonAdapter};

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

type Db = Toydb<AppState, PartitionedJsonAdapter>;

const DB_DIR: &str = "db_data";

fn main() {
    let adapter = PartitionedJsonAdapter::new(DB_DIR);
    let db = Db::open_with_backend(adapter).unwrap();

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
