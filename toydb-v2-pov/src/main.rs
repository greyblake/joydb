use std::{path::PathBuf, sync::{Arc, Mutex}};
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct Db {
    inner: Arc<Mutex<InnerDb>>,
}

impl Db {
    fn new(file_path: impl Into<PathBuf>) -> Self {
        let state = DbState::default();
        let inner = InnerDb {
            file_path: file_path.into(),
            state,
        };
        let inner = Arc::new(Mutex::new(inner));
        Self { inner }
    }

    fn insert<M: Model>(&self, model: M)
    where
        DbState: GetRelation<M>,
    {
        let mut inner = self.inner.lock().unwrap();
        let state = &mut inner.state;
        let relation = <DbState as GetRelation<M>>::get_rel_mut(state);
        relation.push(model);
    }

    fn find<M: Model>(&self, id: &M::Id) -> Option<M>
    where
        M: Clone,
        DbState: GetRelation<M>,
    {
        let inner = self.inner.lock().unwrap();
        let state = &inner.state;
        let relation = <DbState as GetRelation<M>>::get_rel(state);
        relation.iter().find(|m| m.id() == id).cloned()
    }
}

#[derive(Debug)]
struct InnerDb {
    file_path: PathBuf,
    state: DbState,
}

#[derive(Debug, Default)]
struct DbState {
    users: Vec<User>,
    posts: Vec<Post>,
}

#[diagnostic::on_unimplemented(
message = "State `{Self}` does not doest not implement `GetRelation<{M}>`.\nDid you forget to add `{M}` in the state definition?",
note = "Make sure that model `{M}` is listed in the state definition.",
)]
trait GetRelation<M: Model> {
    fn get_rel_mut(&mut self) -> &mut Vec<M>;

    fn get_rel(&self) -> &Vec<M>;
}

impl GetRelation<User> for DbState {
    fn get_rel_mut(&mut self) -> &mut Vec<User> {
        &mut self.users
    }

    fn get_rel(&self) -> &Vec<User> {
        &self.users
    }
}

impl GetRelation<Post> for DbState {
    fn get_rel_mut(&mut self) -> &mut Vec<Post> {
        &mut self.posts
    }

    fn get_rel(&self) -> &Vec<Post> {
        &self.posts
    }
}

// impl GetRelation<Comment> for DbState {
//     fn relation(&mut self) -> &mut Vec<Comment> {
//         unimplemented!()
//     }
// }


pub trait Model: Clone {
    type Id: Debug + Clone + Eq;

    fn id(&self) -> &Self::Id;
}

#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
}

impl Model for User {
    type Id = u64;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

#[derive(Debug, Clone)]
struct Post {
    id: String,
    title: String,
}

impl Model for Post {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

#[derive(Debug, Clone)]
struct Comment {
    id: u64,
    body: String,
}

impl Model for Comment {
    type Id = u64;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

fn main() {
    let db = Db::new("db.json");
    db.insert(User { id: 1, name: "Alice".to_string() });
    db.insert(Post { id: "1".to_string(), title: "Hello".to_string() });
    // db.insert(Comment { id: 1, body: "Nice post".to_string() });

    let user: User = db.find(&1).unwrap();
}
