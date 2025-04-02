use crate::traits::{GetRelation, Model};
use crate::{Relation, State, ToydbError};
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
pub struct Toydb<S: Default + Debug + Serialize + DeserializeOwned + State> {
    inner: Arc<Mutex<InnerToydb<S>>>,
}

// Implement `Clone` manually, otherwise the compile requires a `State: Clone` bound.
// But we deliberately don't want to be the inner state to implement `Clone`.
impl<S: State> Clone for Toydb<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S: State> Toydb<S> {
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
        S: GetRelation<M>,
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
        S: GetRelation<M>,
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
        S: GetRelation<M>,
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
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().count()
    }

    pub fn update<M: Model>(&self, new_model: M) -> Result<(), ToydbError>
    where
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().update(new_model)
    }

    pub fn delete<M: Model>(&self, id: &M::Id) -> Result<Option<M>, ToydbError>
    where
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().delete(id)
    }
}

#[derive(Debug)]
struct InnerToydb<S: State> {
    file_path: PathBuf,
    state: S,
}

impl<S: State> InnerToydb<S> {
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
        let state: S = serde_json::from_str(&content)?;
        Ok(Self { file_path, state })
    }

    fn flush(&mut self) -> Result<(), ToydbError> {
        let content = ::serde_json::to_string_pretty(&self.state)?;
        ::std::fs::write(&self.file_path, content)?;
        // TODO:
        // Add `is_dirty` method on State
        self.state.reset_dirty();
        Ok(())
    }

    fn new(file_path: PathBuf) -> Self {
        let state = S::default();
        InnerToydb { file_path, state }
    }

    fn create(file_path: PathBuf) -> Result<Self, ToydbError> {
        let mut db = Self::new(file_path);
        db.flush()?;
        Ok(db)
    }

    fn is_dirty(&self) -> bool {
        self.state.is_dirty()
    }

    fn get_relation_mut<M: Model>(&mut self) -> &mut Relation<M>
    where
        S: GetRelation<M>,
    {
        let state = &mut self.state;
        <S as GetRelation<M>>::get_rel_mut(state)
    }

    fn get_relation<M: Model>(&self) -> &Relation<M>
    where
        S: GetRelation<M>,
    {
        let state = &self.state;
        <S as GetRelation<M>>::get_rel(state)
    }

    fn insert<M: Model>(&mut self, model: M) -> Result<M, ToydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();
        relation.insert(model)
    }

    fn find<M: Model>(&self, id: &M::Id) -> Result<Option<M>, ToydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        relation.find(id)
    }

    fn all<M: Model>(&self) -> Result<Vec<M>, ToydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        relation.all()
    }

    pub fn count<M: Model>(&self) -> Result<usize, ToydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        relation.count()
    }

    fn update<M: Model>(&mut self, new_model: M) -> Result<(), ToydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();
        relation.update(new_model)
    }

    fn delete<M: Model>(&mut self, id: &M::Id) -> Result<Option<M>, ToydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();
        relation.delete(id)
    }
}

impl<S: State> Drop for InnerToydb<S> {
    fn drop(&mut self) {
        if self.is_dirty() {
            if let Err(err) = self.flush() {
                eprintln!("Failed to flush the database: {}", err);
            }
        }
    }
}
