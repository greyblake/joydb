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

fn main() {
    let mut db = Db::load_or_new("toydb.json").unwrap();

    db.users().insert(User {
        id: 1,
        name: "Alice".to_owned(),
    });

    db.posts().insert(Post {
        id: 1,
        title: "Hello, world!".to_owned(),
    });
}
