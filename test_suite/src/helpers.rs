use crate::database::Database;
use joydb::adapters::JsonAdapter;
use joydb::{JoydbConfig, JoydbMode, SyncPolicy};
use uuid::Uuid;

/// Generate a unique file path for the database file.
pub fn gen_db_file_path() -> String {
    let id = Uuid::new_v4();
    format!("db/test-{}.json", id)
}

/// Open a database and pass it to a closure.
/// The helper takes care of removing the database file after the closure is executed.
pub fn with_open_db<F>(f: F)
where
    F: FnOnce(Database),
{
    let file_path = gen_db_file_path();
    let adapter = JsonAdapter::new(&file_path);
    let config = JoydbConfig {
        mode: JoydbMode::Persistent {
            adapter,
            sync_policy: SyncPolicy::Instant,
        },
    };
    let db = Database::open(config).unwrap();
    f(db);
    std::fs::remove_file(file_path).unwrap();
}
