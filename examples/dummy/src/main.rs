use joydb::{Joydb, Model, SyncMode, adapters::JsonAdapter};
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

type Db = Joydb<AppState, JsonAdapter>;

const DB_FILE: &str = "dummy.json";

fn main() {
    // Delete the file if it exists
    std::fs::remove_file(DB_FILE).ok();

    // Insert some data
    {
        let adapter = JsonAdapter::new(DB_FILE);
        let db = Db::open(adapter, SyncMode::Instant).unwrap();

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
        // DB is flushed automatically on drop
    }

    // Load the data back
    {
        let adapter = JsonAdapter::new(DB_FILE);
        let db = Db::open(adapter, SyncMode::Instant).unwrap();
        let alice: User = db.find(&1).unwrap().unwrap();
        assert_eq!(alice.name, "Alice");

        db.delete::<Post>(&2).unwrap();
    }
}
