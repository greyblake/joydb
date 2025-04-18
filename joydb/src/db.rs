use crate::Model;
use crate::adapters::{Adapter, FromPath};
use crate::{
    JoydbError, Relation,
    state::{GetRelation, State},
};
use std::fmt::Debug;
use std::ops::Drop;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A struct that represents a database.
/// It's thread-safe and can be shared between multiple threads.
///
/// # CRUD operations
///
/// | Operation | Methods                                                                                                  |
/// |-----------|----------------------------------------------------------------------------------------------------------|
/// | Create    | [`insert`](Self::insert), [`upsert`](Self::upsert)                                                       |
/// | Read      | [`get`](Self::get), [`get_all`](Self::get_all), [`get_all_by`](Self::get_all_by), [`count`](Self::count) |
/// | Update    | [`update`](Self::update), [`upsert`](Self::upsert)                                                       |
/// | Delete    | [`delete`](Self::delete), [`delete_all_by`](Self::delete_all_by)                                         |
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

impl<S: State, A: Adapter + FromPath> Joydb<S, A> {
    /// Opens a database from the given file/directory path.
    /// If the database does not exist, it will be created.
    ///
    /// # Example
    /// TODO example
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, JoydbError> {
        let adapter = A::from_path(path);
        let config = JoydbConfig {
            mode: JoydbMode::Persistent {
                adapter,
                sync_policy: SyncPolicy::Instant,
            },
        };
        Self::open_with_config(config)
    }
}

impl<S: State, A: Adapter> Joydb<S, A> {
    /// Creates a new in-memory database.
    /// This database is not persisted to the file system.
    /// This is intended to be used mostly in tests.
    pub fn new_in_memory() -> Result<Self, JoydbError> {
        let config = JoydbConfig {
            mode: JoydbMode::InMemory,
        };
        Self::open_with_config(config)
    }

    pub fn open_with_config(config: JoydbConfig<A>) -> Result<Self, JoydbError> {
        let maybe_sync_policy = config.sync_policy();

        let inner: InnerJoydb<S, A> = InnerJoydb::open_with_config(config)?;
        let arc_inner = Arc::new(Mutex::new(inner));

        if let Some(SyncPolicy::Periodic(duration)) = maybe_sync_policy {
            let weak_inner_db = Arc::downgrade(&arc_inner);
            spawn_periodic_sync_thread(duration, weak_inner_db);
        }

        Ok(Self { inner: arc_inner })
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
    pub fn get<M: Model>(&self, id: &M::Id) -> Result<Option<M>, JoydbError>
    where
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().get(id)
    }

    /// Returns all records that corresponds to the model type.
    /// The order of the records is not guaranteed and is a subject to change in the future versions.
    ///
    /// # Complexity
    /// O(n)
    ///
    /// # Example
    /// TODO
    pub fn get_all<M: Model>(&self) -> Result<Vec<M>, JoydbError>
    where
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().get_all()
    }

    /// Return all records that match the predicate.
    pub fn get_all_by<M, F>(&self, predicate: F) -> Result<Vec<M>, JoydbError>
    where
        M: Model,
        S: GetRelation<M>,
        F: Fn(&M) -> bool,
    {
        self.inner.lock().unwrap().get_all_by(predicate)
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

    pub fn update<M: Model>(&self, new_record: &M) -> Result<(), JoydbError>
    where
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().update(new_record)
    }

    /// Upserts a record.
    /// If the record with the same id already exists, it will be updated.
    /// Otherwise, it will be inserted.
    pub fn upsert<M: Model>(&self, record: &M) -> Result<(), JoydbError>
    where
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().upsert(record)
    }

    pub fn delete<M: Model>(&self, id: &M::Id) -> Result<Option<M>, JoydbError>
    where
        S: GetRelation<M>,
    {
        self.inner.lock().unwrap().delete(id)
    }

    /// Deletes all records that match the predicate.
    pub fn delete_all_by<M, F>(&self, predicate: F) -> Result<Vec<M>, JoydbError>
    where
        M: Model,
        S: GetRelation<M>,
        F: Fn(&M) -> bool,
    {
        self.inner.lock().unwrap().delete_all_by(predicate)
    }

    /// Flushes the state to the file system.
    /// If there are any unsaved changes the corresponding file(s) will be rewritten from scratch.
    /// This method is also always called automatically on drop.
    pub fn flush(&self) -> Result<(), JoydbError> {
        self.inner.lock().unwrap().flush()
    }
}

#[derive(Debug)]
struct InnerJoydb<S: State, A: Adapter> {
    state: S,
    mode: JoydbMode<A>,
}

impl<S: State, A: Adapter> InnerJoydb<S, A> {
    fn open_with_config(config: JoydbConfig<A>) -> Result<Self, JoydbError> {
        let JoydbConfig { mode } = config;

        // Get the initial state
        let state = match &mode {
            JoydbMode::Persistent {
                adapter,
                sync_policy: _,
            } => adapter.load_state::<S>()?,
            JoydbMode::InMemory => S::default(),
        };

        Ok(Self { state, mode })
    }

    /// Write data to the file system if there are unsaved changes.
    fn flush(&mut self) -> Result<(), JoydbError> {
        if self.is_dirty() {
            self.write_state()?;
            self.state.reset_dirty();
        }
        Ok(())
    }

    fn write_state(&mut self) -> Result<(), JoydbError> {
        match &self.mode {
            JoydbMode::Persistent {
                adapter,
                sync_policy: _,
            } => adapter.write_state(&self.state),
            JoydbMode::InMemory => {
                // Do nothing
                Ok(())
            }
        }
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

    fn get<M: Model>(&self, id: &M::Id) -> Result<Option<M>, JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        relation.get(id)
    }

    fn get_all<M: Model>(&self) -> Result<Vec<M>, JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        relation.get_all()
    }

    /// Return all records that match the predicate.
    pub(crate) fn get_all_by<M, F>(&self, predicate: F) -> Result<Vec<M>, JoydbError>
    where
        M: Model,
        S: GetRelation<M>,
        F: Fn(&M) -> bool,
    {
        let relation = self.get_relation::<M>();
        relation.get_all_by(predicate)
    }

    pub fn count<M: Model>(&self) -> Result<usize, JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        relation.count()
    }

    fn update<M: Model>(&mut self, new_record: &M) -> Result<(), JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();
        relation.update(new_record)?;
        self.after_change()?;
        Ok(())
    }

    fn upsert<M: Model>(&mut self, record: &M) -> Result<(), JoydbError>
    where
        S: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();
        relation.upsert(record)?;
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

    pub fn delete_all_by<M, F>(&mut self, predicate: F) -> Result<Vec<M>, JoydbError>
    where
        M: Model,
        S: GetRelation<M>,
        F: Fn(&M) -> bool,
    {
        let relation = self.get_relation_mut::<M>();
        let deleted_records = relation.delete_all_by(predicate)?;
        if !deleted_records.is_empty() {
            self.after_change()?;
        }
        Ok(deleted_records)
    }

    /// Hook which is called every time after database state has changed.
    fn after_change(&mut self) -> Result<(), JoydbError> {
        if self.mode.is_instant_sync_policy() {
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
pub enum SyncPolicy {
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
}

#[derive(Debug)]
pub struct JoydbConfig<A: Adapter> {
    pub mode: JoydbMode<A>,
}

impl<A: Adapter> JoydbConfig<A> {
    fn sync_policy(&self) -> Option<SyncPolicy> {
        match &self.mode {
            JoydbMode::Persistent { sync_policy, .. } => Some(*sync_policy),
            JoydbMode::InMemory => None,
        }
    }
}

/// The mode of the database.
/// This is used to specify how the database should be opened.
#[derive(Debug)]
pub enum JoydbMode<A: Adapter> {
    /// The data are flushed to the file system.
    Persistent {
        /// The adapter used to read/write the data (e.g. JSON, CSV, etc.)
        adapter: A,

        /// Specifies when data must be flushed to the file system.
        sync_policy: SyncPolicy,
    },
    /// The data are never flushed to the file system. Even when [Joydb::flush] is explicitly
    /// called.
    /// With this mode, Joydb acts like in-memory-only database and this mode is mostly intended
    /// for unit tests.
    InMemory,
}

impl<A: Adapter> JoydbMode<A> {
    fn is_instant_sync_policy(&self) -> bool {
        match self {
            JoydbMode::Persistent { sync_policy, .. } => *sync_policy == SyncPolicy::Instant,
            JoydbMode::InMemory => false,
        }
    }
}

/// Spawns a thread that periodically flushes the database.
/// The thread owns a weak reference to the database, and runs until the database is dropped.
/// This is used only when the [SyncPolicy] is set to [`Periodic`](SyncPolicy::Periodic).
fn spawn_periodic_sync_thread<S: State, A: Adapter>(
    interval: Duration,
    weak_inner_db: std::sync::Weak<Mutex<InnerJoydb<S, A>>>,
) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(interval);
            if let Some(inner) = weak_inner_db.upgrade() {
                inner
                    .lock()
                    .expect("Failed to lock the Joydb database from the background thread")
                    .flush()
                    .expect("Failed to flush the Joydb database from the background thread");
            } else {
                break;
            }
        }
    });
}
