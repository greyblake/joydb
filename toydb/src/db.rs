use std::{path::PathBuf, sync::{Arc, Mutex}};
use std::fmt::Debug;
use std::ops::Drop;
use crate::traits::{Model, GetRelation};
use serde::{de::DeserializeOwned, Serialize};
use crate::StorageError;

#[derive(Debug, Clone)]
pub struct Toydb<State: Default + Debug + Serialize + DeserializeOwned> {
    inner: Arc<Mutex<InnerToydb<State>>>,
}

impl<State: Default + Debug + Serialize + DeserializeOwned> Toydb<State> {
    pub fn open(file_path: impl Into<::std::path::PathBuf>) -> Result<Self, StorageError> {
        let file_path = file_path.into();
        let inner = InnerToydb::open(file_path)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    pub fn insert<M: Model>(&self, model: M)
    where
        State: GetRelation<M>,
    {
        let mut inner = self.inner.lock().unwrap();
        let state = &mut inner.state;
        let relation = <State as GetRelation<M>>::get_rel_mut(state);

        relation.push(model);
        inner.changes_count += 1;
    }

    pub fn find<M: Model>(&self, id: &M::Id) -> Option<M>
    where
        M: Clone,
        State: GetRelation<M>,
    {
        let inner = self.inner.lock().unwrap();
        let state = &inner.state;
        let relation = <State as GetRelation<M>>::get_rel(state);

        relation.iter().find(|m| m.id() == id).cloned()
    }

    pub fn update<M: Model>(&self, new_model: M)
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
        inner.changes_count += 1;
    }

    pub fn delete<M: Model>(&self, id: &M::Id) -> Option<M>
    where
        State: GetRelation<M>,
    {
        let mut inner = self.inner.lock().unwrap();
        let state = &mut inner.state;
        let relation = <State as GetRelation<M>>::get_rel_mut(state);

        let index = relation.iter().position(|m| m.id() == id);
        if let Some(index) = index {
            let record = relation.remove(index);
            inner.changes_count += 1;
            Some(record)
        } else {
            None
        }
    }

    // TODO:
    //
    // Getters:
    // - find_all_by(predicate) -> Vec<M>
    // - all() -> Vec<M>
}

#[derive(Debug)]
struct InnerToydb<State: Default + Debug + Serialize + DeserializeOwned> {
    file_path: PathBuf,
    state: State,
    changes_count: u64,
}

impl<State: Default + Debug + Serialize + DeserializeOwned> InnerToydb<State> {
    fn open(file_path: PathBuf) -> Result<Self, StorageError> {
        let path = ::std::path::Path::new(&file_path);
        if path.exists() {
            if path.is_file() {
                Self::load(file_path)
            } else {
                Err(StorageError::NotFile(file_path))
            }
        } else {
            Self::create(file_path)
        }
    }

    fn load(file_path: PathBuf) -> Result<Self, StorageError> {
        let content = std::fs::read_to_string(&file_path)?;
        let state: State = serde_json::from_str(&content)?;
        Ok(Self {
            file_path,
            state,
            changes_count: 0,
        })
    }

    fn flush(&mut self) -> Result<(), StorageError> {
        let content = ::serde_json::to_string_pretty(&self.state)?;
        ::std::fs::write(&self.file_path, content)?;
        self.changes_count = 0;
        Ok(())
    }

    fn new(file_path: PathBuf) -> Self {
        let state = State::default();
        InnerToydb {
            file_path: file_path,
            state,
            changes_count: 0,
        }
    }

    fn create(file_path: PathBuf) -> Result<Self, StorageError> {
        let mut db = Self::new(file_path);
        db.flush()?;
        Ok(db)
    }

    fn is_dirty(&self) -> bool {
        self.changes_count > 0
    }
}


impl<State: Default + Debug + Serialize + DeserializeOwned> Drop for InnerToydb<State> {
    fn drop(&mut self) {
        if self.is_dirty() {
            if let Err(err) = self.flush() {
                eprintln!("Failed to flush the database: {}", err);
            }
        }
    }
}

