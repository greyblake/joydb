use crate::ToydbError;
use crate::traits::{GetRelation, Model};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::ops::Drop;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// A struct that represents a database.
/// It's thread-safe and can be shared between multiple threads.
///
/// # CRUD operations
///
/// | Operation | Methods                                                          |
/// |-----------|------------------------------------------------------------------|
/// | Create    | [`insert`](Self::insert)                                         |
/// | Read      | [`find`](Self::find), [`all`](Self::all), [`count`](Self::count) |
/// | Update    | [`update`](Self::update)                                         |
/// | Delete    | [`delete`](Self::delete)                                         |
///
#[derive(Debug)]
pub struct Toydb<State: Default + Debug + Serialize + DeserializeOwned> {
    inner: Arc<Mutex<InnerToydb<State>>>,
}

// Implement `Clone` manually, otherwise the compile requires a `State: Clone` bound.
// But we deliberately don't want to be the inner state to implement `Clone`.
impl<State: Default + Debug + Serialize + DeserializeOwned> Clone for Toydb<State> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<State: Default + Debug + Serialize + DeserializeOwned> Toydb<State> {
    pub fn open(file_path: impl Into<::std::path::PathBuf>) -> Result<Self, ToydbError> {
        let file_path = file_path.into();
        let inner = InnerToydb::open(file_path)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    /// Inserts a new record.
    /// Returns the inserted record.
    ///
    /// # Errors
    /// Returns an error if the record with the same id already exists.
    ///
    /// # Complexity
    /// O(n)
    ///
    /// # Example
    /// TODO
    pub fn insert<M: Model>(&self, model: M) -> Result<M, ToydbError>
    where
        State: GetRelation<M>,
    {
        self.inner.lock().unwrap().insert(model)
    }

    /// Finds a record by its id.
    /// Returns `None` if the record is not found.
    ///
    /// # Complexity
    /// O(n)
    ///
    /// # Example
    ///
    /// TODO
    pub fn find<M: Model>(&self, id: &M::Id) -> Result<Option<M>, ToydbError>
    where
        State: GetRelation<M>,
    {
        self.inner.lock().unwrap().find(id)
    }

    /// Returns all records that corresponds to the model type.
    /// The order of the records is the same as they were inserted.
    ///
    /// # Complexity
    /// O(n)
    ///
    /// # Example
    /// TODO
    pub fn all<M: Model>(&self) -> Result<Vec<M>, ToydbError>
    where
        State: GetRelation<M>,
    {
        self.inner.lock().unwrap().all()
    }

    /// Returns the number of records that corresponds to the model type.
    ///
    /// # Complexity
    /// O(1)
    ///
    /// # Errors
    ///
    /// No real errors are expected to happen.
    /// However, `Result<usize, ToydbError>` is used to keep the API consistent with other methods
    /// and to make the user treat interaction with the database as fallible operations.
    ///
    /// # Example
    /// TODO
    ///
    pub fn count<M: Model>(&self) -> Result<usize, ToydbError>
    where
        State: GetRelation<M>,
    {
        self.inner.lock().unwrap().count()
    }

    pub fn update<M: Model>(&self, new_model: M) -> Result<(), ToydbError>
    where
        State: GetRelation<M>,
    {
        self.inner.lock().unwrap().update(new_model)
    }

    pub fn delete<M: Model>(&self, id: &M::Id) -> Result<Option<M>, ToydbError>
    where
        State: GetRelation<M>,
    {
        self.inner.lock().unwrap().delete(id)
    }
}

#[derive(Debug)]
struct InnerToydb<State: Default + Debug + Serialize + DeserializeOwned> {
    file_path: PathBuf,
    state: State,
    changes_count: u64,
}

impl<State: Default + Debug + Serialize + DeserializeOwned> InnerToydb<State> {
    fn open(file_path: PathBuf) -> Result<Self, ToydbError> {
        let path = ::std::path::Path::new(&file_path);
        if path.exists() {
            if path.is_file() {
                Self::load(file_path)
            } else {
                Err(ToydbError::NotFile(file_path))
            }
        } else {
            Self::create(file_path)
        }
    }

    fn load(file_path: PathBuf) -> Result<Self, ToydbError> {
        let content = std::fs::read_to_string(&file_path)?;
        let state: State = serde_json::from_str(&content)?;
        Ok(Self {
            file_path,
            state,
            changes_count: 0,
        })
    }

    fn flush(&mut self) -> Result<(), ToydbError> {
        let content = ::serde_json::to_string_pretty(&self.state)?;
        ::std::fs::write(&self.file_path, content)?;
        self.changes_count = 0;
        Ok(())
    }

    fn new(file_path: PathBuf) -> Self {
        let state = State::default();
        InnerToydb {
            file_path,
            state,
            changes_count: 0,
        }
    }

    fn create(file_path: PathBuf) -> Result<Self, ToydbError> {
        let mut db = Self::new(file_path);
        db.flush()?;
        Ok(db)
    }

    fn is_dirty(&self) -> bool {
        self.changes_count > 0
    }

    fn get_relation_mut<M: Model>(&mut self) -> &mut Vec<M>
    where
        State: GetRelation<M>,
    {
        let state = &mut self.state;
        <State as GetRelation<M>>::get_rel_mut(state)
    }

    fn get_relation<M: Model>(&self) -> &[M]
    where
        State: GetRelation<M>,
    {
        let state = &self.state;
        <State as GetRelation<M>>::get_rel(state)
    }

    fn insert<M: Model>(&mut self, model: M) -> Result<M, ToydbError>
    where
        State: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();

        let id = model.id();
        let is_duplicated = relation.iter().find(|m| m.id() == id).is_some();
        if is_duplicated {
            return Err(ToydbError::DuplicatedId {
                id: format!("{:?}", id),
                model_name: base_type_name::<M>().to_owned(),
            });
        } else {
            relation.push(model.clone());
            self.changes_count += 1;
            Ok(model)
        }
    }

    fn find<M: Model>(&self, id: &M::Id) -> Result<Option<M>, ToydbError>
    where
        State: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        let maybe_record = relation.iter().find(|m| m.id() == id).cloned();
        Ok(maybe_record)
    }

    fn all<M: Model>(&self) -> Result<Vec<M>, ToydbError>
    where
        State: GetRelation<M>,
    {
        let records = self.get_relation::<M>().to_vec();
        Ok(records)
    }

    pub fn count<M: Model>(&self) -> Result<usize, ToydbError>
    where
        State: GetRelation<M>,
    {
        Ok(self.get_relation::<M>().len())
    }

    fn update<M: Model>(&mut self, new_model: M) -> Result<(), ToydbError>
    where
        State: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();

        let id = new_model.id();
        if let Some(m) = relation.iter_mut().find(|m| m.id() == id) {
            *m = new_model;
            self.changes_count += 1;
            Ok(())
        } else {
            // TODO: Return error?
            panic!(
                "Model {} not found by id = {:?}",
                std::any::type_name::<M>(),
                id
            );
        }
    }

    fn delete<M: Model>(&mut self, id: &M::Id) -> Result<Option<M>, ToydbError>
    where
        State: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();

        let index = relation.iter().position(|m| m.id() == id);
        if let Some(index) = index {
            let record = relation.remove(index);
            self.changes_count += 1;
            Ok(Some(record))
        } else {
            Ok(None)
        }
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

fn base_type_name<T>() -> &'static str {
    let full_name = std::any::type_name::<T>();
    full_name.split_terminator("::").last().unwrap()
}
