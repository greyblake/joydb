use serde::{Deserialize, Serialize};
use toydb::Model;

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

toydb::define_storage! {
    Db,
    users: User,
    posts: Post,
}

const DB_FILE: &str = "dummy.json";

fn main() {
    // Delete the file if it exists
    std::fs::remove_file(DB_FILE).ok();

    // Insert some data
    {
        let mut db = Db::load_or_create(DB_FILE).unwrap();

        db.users().insert(User {
            id: 1,
            name: "Alice".to_owned(),
        });
        db.users().insert(User {
            id: 2,
            name: "Bob".to_owned(),
        });

        db.posts().insert(Post {
            id: 1,
            title: "Hello, world!".to_owned(),
        });
        // NOTE: DB is flushed automatically on drop
    }

    // Load the data back
    {
        let mut db = Db::load_or_create(DB_FILE).unwrap();
        let alice = db.users().get_by_id(1).unwrap();
        assert_eq!(alice.name, "Alice");
    }
}
