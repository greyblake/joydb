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

joydb::define_state! {
    AppState,
    models: [User, Post],
}

type Db = Joydb<AppState, JsonAdapter>;

const DATA_FILE: &str = "data.json";

fn main() {
    // Delete the file if it exists
    std::fs::remove_file(DATA_FILE).ok();

    // Insert some data
    {
        let db = Db::open(DATA_FILE).unwrap();

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
        let db = Db::open(DATA_FILE).unwrap();
        let alice: User = db.get(&1).unwrap().unwrap();
        assert_eq!(alice.name, "Alice");

        db.delete::<Post>(&2).unwrap();
    }
}
