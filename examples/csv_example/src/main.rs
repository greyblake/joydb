use joydb::{Joydb, Model, adapters::CsvAdapter};
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

type Db = Joydb<AppState, CsvAdapter>;

const DB_DIR: &str = "data";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Remove old directory with data if it exists
    {
        std::fs::remove_dir_all(DB_DIR).ok();
    }

    // Write something
    {
        let adapter = CsvAdapter::new(DB_DIR);
        let db = Db::open_with_adapter(adapter)?;

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

    // Read something
    {
        let adapter = CsvAdapter::new(DB_DIR);
        let db = Db::open_with_adapter(adapter)?;

        let alice = db.find::<User>(&1)?.unwrap();
        assert_eq!(alice.name, "Alice");

        let bob = db.find::<User>(&2)?.unwrap();
        assert_eq!(bob.name, "Bob");

        let post = db.find::<Post>(&1)?.unwrap();
        assert_eq!(post.title, "Hello, world!");
    }

    Ok(())
}
