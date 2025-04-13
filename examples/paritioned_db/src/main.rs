use joydb::{Joydb, Model, adapters::PartitionedJsonAdapter};
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

joydb::define_state! {
    AppState,
    models: [User, Post],
}

type Db = Joydb<AppState, PartitionedJsonAdapter>;

const DB_DIR: &str = "db_data";

fn main() {
    let db = Db::open(DB_DIR).unwrap();

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
