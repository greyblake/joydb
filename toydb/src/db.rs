use crate::adapters::{Backend, UnifiedAdapter};
use crate::{Model, RelationAdapter};
use crate::{
    Relation, ToydbError,
    state::{GetRelation, State},
};
use std::fmt::Debug;
use std::marker::PhantomData;
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
pub struct Toydb<S: State, UA: UnifiedAdapter, RA: RelationAdapter> {
    inner: Arc<Mutex<InnerToydb<S, UA, RA>>>,
}

// Implement `Clone` manually, otherwise the compile requires a `State: Clone` bound.
// But we deliberately don't want to be the inner state to implement `Clone`.
impl<S: State, UA: UnifiedAdapter, RA: RelationAdapter> Clone for Toydb<S, UA, RA> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S: State, UA: UnifiedAdapter, RA: RelationAdapter> Toydb<S, UA, RA> {
    // TODO: Pass backend here as a parameter!
    pub fn open_with_backend(
        backend: Backend<UA, RA>,
        file_path: impl Into<::std::path::PathBuf>,
    ) -> Result<Self, ToydbError> {
        let file_path = file_path.into();
        let inner: InnerToydb<S, UA, RA> = InnerToydb::open_with_backend(backend, file_path)?;
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
    pub fn insert<M: Model>(&self, model: &M) -> Result<(), ToydbError>
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
struct InnerToydb<S: State, UA: UnifiedAdapter, RA: RelationAdapter> {
    file_path: PathBuf,
    state: S,
    backend: Backend<UA, RA>,
}

impl<S: State, UA: UnifiedAdapter, RA: RelationAdapter> InnerToydb<S, UA, RA> {
    fn open_with_backend(backend: Backend<UA, RA>, file_path: PathBuf) -> Result<Self, ToydbError> {
        let path = ::std::path::Path::new(&file_path);
        if path.exists() {
            if path.is_file() {
                Self::load(backend, file_path)
            } else {
                Err(ToydbError::NotFile(file_path))
            }
        } else {
            Self::create_with_backend(backend, file_path)
        }
    }

    fn load(backend: Backend<UA, RA>, file_path: PathBuf) -> Result<Self, ToydbError> {
        let state: S = UA::read(&file_path)?;
        Ok(Self {
            file_path,
            state,
            backend,
        })
    }

    /// Write data to the file system if there are unsaved changes.
    fn flush(&mut self) -> Result<(), ToydbError> {
        if self.is_dirty() {
            self.save()?;
            self.state.reset_dirty();
        }
        Ok(())
    }

    fn new(backend: Backend<UA, RA>, file_path: PathBuf) -> Self {
        let state = S::default();
        InnerToydb {
            file_path,
            state,
            backend,
        }
    }

    fn create_with_backend(
        backend: Backend<UA, RA>,
        file_path: PathBuf,
    ) -> Result<Self, ToydbError> {
        let mut db = Self::new(backend, file_path);
        db.save()?;
        Ok(db)
    }

    fn save(&mut self) -> Result<(), ToydbError> {
        UA::write(&self.file_path, &self.state)
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

    fn insert<M: Model>(&mut self, model: &M) -> Result<(), ToydbError>
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

impl<S: State, UA: UnifiedAdapter, RA: RelationAdapter> Drop for InnerToydb<S, UA, RA> {
    fn drop(&mut self) {
        if let Err(err) = self.flush() {
            eprintln!("Failed to flush the database: {}", err);
        }
    }
}
