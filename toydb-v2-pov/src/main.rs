use std::{path::PathBuf, sync::{Arc, Mutex}};
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct Toydb<State: Default + Debug> {
    inner: Arc<Mutex<InnerToydb<State>>>,
}

impl<State: Default + Debug> Toydb<State> {
    fn new(file_path: impl Into<PathBuf>) -> Self {
        let state = State::default();
        let inner = InnerToydb {
            file_path: file_path.into(),
            state,
            dirty_changes_count: 0,
        };
        let inner = Arc::new(Mutex::new(inner));
        Self { inner }
    }

    fn insert<M: Model>(&self, model: M)
    where
        State: GetRelation<M>,
    {
        let mut inner = self.inner.lock().unwrap();
        let state = &mut inner.state;
        let relation = <State as GetRelation<M>>::get_rel_mut(state);
        relation.push(model);
        inner.dirty_changes_count += 1;
    }

    fn find<M: Model>(&self, id: &M::Id) -> Option<M>
    where
        M: Clone,
        State: GetRelation<M>,
    {
        let inner = self.inner.lock().unwrap();
        let state = &inner.state;
        let relation = <State as GetRelation<M>>::get_rel(state);
        relation.iter().find(|m| m.id() == id).cloned()
    }

    fn update<M: Model>(&self, new_model: M)
    where
        State: GetRelation<M>,
    {
        let mut inner = self.inner.lock().unwrap();
        let state = &mut inner.state;
        let relation = <State as GetRelation<M>>::get_rel_mut(state);
        let id = new_model.id();
        if let Some(m) = relation.iter_mut().find(|m| m.id() == id) {
            *m = new_model;
        } else {
            // TODO: Return error?
            panic!("Model {} not found by id = {:?}", std::any::type_name::<M>(), id);
        }
        inner.dirty_changes_count += 1;
    }

    // TODO:
    // - delete(&id) -> Option<M>
    //
    // Getters:
    // - find_all_by(predicate) -> Vec<M>
    // - all() -> Vec<M>

}

#[derive(Debug)]
struct InnerToydb<State: Default + Debug> {
    file_path: PathBuf,
    state: State,
    dirty_changes_count: u64,
}

#[derive(Debug, Default)]
struct AppState {
    // TODO: For serde, load empty if property is missing
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

impl GetRelation<User> for AppState {
    fn get_rel_mut(&mut self) -> &mut Vec<User> {
        &mut self.users
    }

    fn get_rel(&self) -> &Vec<User> {
        &self.users
    }
}

impl GetRelation<Post> for AppState {
    fn get_rel_mut(&mut self) -> &mut Vec<Post> {
        &mut self.posts
    }

    fn get_rel(&self) -> &Vec<Post> {
        &self.posts
    }
}


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


// State type
// List of model types
// define_toydb! {
//     AppState { User, Post, Comment, }
// }

type AppDb = Toydb<AppState>;

fn main() {
    let db = AppDb::new("db.json");
    db.insert(User { id: 1, name: "Alice".to_string() });
    db.insert(Post { id: "1".to_string(), title: "Hello".to_string() });
    // db.insert(Comment { id: 1, body: "Nice post".to_string() });

    let user: User = db.find(&1).unwrap();
    assert_eq!(user.name, "Alice");

    db.update(User { id: 1, name: "Former Alice".to_string() });
    assert_eq!(db.find::<User>(&1).unwrap().name, "Former Alice");


    // let users = db.all::<User>();
}
