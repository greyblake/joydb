use crate::Model;
use crate::adapters::Adapter;
use crate::{
    JoydbError, Relation,
    state::{GetRelation, State},
};
use std::time::Duration;
use std::fmt::Debug;
use std::ops::Drop;
use std::sync::{Arc, Mutex};

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
pub struct Joydb<S: State, A: Adapter> {
    inner: Arc<Mutex<InnerJoydb<S, A>>>,
}

// Implement `Clone` manually, otherwise the compile requires a `State: Clone` bound.
// But we deliberately don't want to be the inner state to implement `Clone`.
impl<S: State, A: Adapter> Clone for Joydb<S, A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S: State, A: Adapter> Joydb<S, A> {
    pub fn open(adapter: A, sync_mode: SyncMode) -> Result<Self, JoydbError> {
        let inner: InnerJoydb<S, A> = InnerJoydb::open(adapter, sync_mode)?;
        let arc_inner = Arc::new(Mutex::new(inner));

        if let SyncMode::Periodic(duration) = sync_mode {
            let weak_inner_db = Arc::downgrade(&arc_inner);
            spawn_periodic_sync_thread(duration, weak_inner_db);
        }

        Ok(Self {
            inner: arc_inner,
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
    pub fn insert<M: Model>(&self, model: &M) -> Result<(), JoydbError>
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
    pub fn find<M: Model>(&self, id: &M::Id) -> Result<Option<M>, JoydbError>
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
    pub fn all<M: Model>(&self) -> Result<Vec<M>, JoydbError>
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
    /// However, `Result<usize, JoydbError>` is used to keep the API consistent with other methods
    /// and to make the user treat interaction with the database as fallible operations.
    ///
    /// # Example
    /// TODO
    ///
    pub fn count<M: Model>(&self) -> Result<usize, JoydbError>
    where
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().count()
    }

    pub fn update<M: Model>(&self, new_model: M) -> Result<(), JoydbError>
    where
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().update(new_model)
    }

    pub fn delete<M: Model>(&self, id: &M::Id) -> Result<Option<M>, JoydbError>
    where
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().delete(id)
    }
}

#[derive(Debug)]
struct InnerJoydb<S: State, A: Adapter> {
    state: S,
    adapter: A,
    sync_mode: SyncMode,
}

impl<S: State, A: Adapter> InnerJoydb<S, A> {
    fn open(adapter: A, sync_mode: SyncMode) -> Result<Self, JoydbError> {
        let state = adapter.load_state::<S>()?;
        Ok(Self {
            state,
            adapter,
            sync_mode,
        })
    }

    /// Write data to the file system if there are unsaved changes.
    fn flush(&mut self) -> Result<(), JoydbError> {
        if self.is_dirty() {
            self.save()?;
            self.state.reset_dirty();
        }
        Ok(())
    }

    fn save(&mut self) -> Result<(), JoydbError> {
        self.adapter.write_state(&self.state)
    }

    fn is_dirty(&self) -> bool {
        self.state.is_dirty()
    }

    fn get_relation_mut<M: Model>(&mut self) -> &mut Relation<M>
    where
        S: GetRelation<M>,
    {
        let state = &mut self.state;
        <S as GetRelation<M>>::get_relation_mut(state)
    }

    fn get_relation<M: Model>(&self) -> &Relation<M>
    where
        S: GetRelation<M>,
    {
        let state = &self.state;
        <S as GetRelation<M>>::get_relation(state)
    }

    fn insert<M: Model>(&mut self, model: &M) -> Result<(), JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();
        relation.insert(model)?;
        self.after_change()?;
        Ok(())
    }

    fn find<M: Model>(&self, id: &M::Id) -> Result<Option<M>, JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        relation.find(id)
    }

    fn all<M: Model>(&self) -> Result<Vec<M>, JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        relation.all()
    }

    pub fn count<M: Model>(&self) -> Result<usize, JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        relation.count()
    }

    fn update<M: Model>(&mut self, new_model: M) -> Result<(), JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();
        relation.update(new_model)?;
        self.after_change()?;
        Ok(())
    }

    fn delete<M: Model>(&mut self, id: &M::Id) -> Result<Option<M>, JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();
        let maybe_deleted_record = relation.delete(id)?;
        if maybe_deleted_record.is_some() {
            self.after_change()?;
        }
        Ok(maybe_deleted_record)
    }

    /// Hook which  is called every time after database state has changed.
    fn after_change(&mut self) -> Result<(), JoydbError> {
        if self.sync_mode == SyncMode::Instant {
            self.flush()?;
        }
        Ok(())
    }
}

impl<S: State, A: Adapter> Drop for InnerJoydb<S, A> {
    fn drop(&mut self) {
        if let Err(err) = self.flush() {
            eprintln!("Failed to flush the database: {}", err);
        }
    }
}

/// Specifies how and when the database should be synchronized with the file system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyncMode {
    /// The data are flushed to the file system instantly with every mutable operation.
    /// This is the default mode.
    /// This mode is the slowest, but the safest.
    Instant,

    /// The data are flushed to the file system periodically by a thread
    /// that runs in the background.
    Periodic(Duration),

    /// The data are flushed to the file system manually when the [Joydb::flush] method is called.
    /// The only exception is on drop, which always flushes the data.
    Manual,

    // /// The data are never flushed to the file system. Even when [Joydb::flush] is explicitly
    // /// called.
    // /// With this mode, Joydb acts like in-memory-only database and this mode is mostly intended
    // /// for unit tests.
    // TODO: This needs to be better thought out, because essentially it does not require adapter
    // and file/dir path specified.
    // Never,
}


/// The thread is running in the background until the database is dropped.
fn spawn_periodic_sync_thread<S: State, A: Adapter>(
    interval: Duration,
    weak_inner_db: std::sync::Weak<Mutex<InnerJoydb<S, A>>>,
) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(interval);
            if let Some(inner) = weak_inner_db.upgrade() {
             inner.lock()
                 .expect("Failed to lock the Joydb database from the background thread")
                 .flush()
                 .expect("Failed to flush the Joydb database from the background thread");
            } else {
                break;
            }
        }
    });
}
