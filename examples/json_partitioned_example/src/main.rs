use joydb::{Joydb, Model, adapters::JsonPartitionedAdapter};
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

type Db = Joydb<AppState, JsonPartitionedAdapter>;

const DATA_DIR: &str = "data";

fn main() {
    // Delete the file if it exists
    std::fs::remove_file(DATA_DIR).ok();

    // Insert some data
    {
        let db = Db::open(DATA_DIR).unwrap();

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
        let db = Db::open(DATA_DIR).unwrap();
        let alice: User = db.find(&1).unwrap().unwrap();
        assert_eq!(alice.name, "Alice");

        db.delete::<Post>(&2).unwrap();
    }
}
