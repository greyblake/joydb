use crate::adapters::{Backend, UnifiedAdapter};
use crate::{Model, NeverAdapter, PartitionedAdapter};
use crate::{
    Relation, ToydbError,
    state::{GetRelation, State},
};
use std::fmt::Debug;
use std::ops::Drop;
use std::sync::{Arc, Mutex};

pub type UnifiedDb<S, UA> = Toydb<S, UA, NeverAdapter>;
pub type PartitionedDb<S, PA> = Toydb<S, NeverAdapter, PA>;

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
pub struct Toydb<S: State, UA: UnifiedAdapter, PA: PartitionedAdapter> {
    inner: Arc<Mutex<InnerToydb<S, UA, PA>>>,
}

// Implement `Clone` manually, otherwise the compile requires a `State: Clone` bound.
// But we deliberately don't want to be the inner state to implement `Clone`.
impl<S: State, UA: UnifiedAdapter, PA: PartitionedAdapter> Clone for Toydb<S, UA, PA> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S: State, UA: UnifiedAdapter, PA: PartitionedAdapter> Toydb<S, UA, PA> {
    pub fn open_with_backend(backend: Backend<UA, PA>) -> Result<Self, ToydbError> {
        let inner: InnerToydb<S, UA, PA> = InnerToydb::open_with_backend(backend)?;
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
struct InnerToydb<S: State, UA: UnifiedAdapter, PA: PartitionedAdapter> {
    state: S,
    backend: Backend<UA, PA>,
}

impl<S: State, UA: UnifiedAdapter, PA: PartitionedAdapter> InnerToydb<S, UA, PA> {
    fn open_with_backend(backend: Backend<UA, PA>) -> Result<Self, ToydbError> {
        let state = backend.init_state::<S>()?;
        Ok(Self { state, backend })
    }

    /// Write data to the file system if there are unsaved changes.
    fn flush(&mut self) -> Result<(), ToydbError> {
        if self.is_dirty() {
            self.save()?;
            self.state.reset_dirty();
        }
        Ok(())
    }

    fn save(&mut self) -> Result<(), ToydbError> {
        self.state.write_with_backend(&self.backend)
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

impl<S: State, UA: UnifiedAdapter, PA: PartitionedAdapter> Drop for InnerToydb<S, UA, PA> {
    fn drop(&mut self) {
        if let Err(err) = self.flush() {
            eprintln!("Failed to flush the database: {}", err);
        }
    }
}
