use crate::database::Database;
use uuid::Uuid;

/// Directory with test data files.
const DATA_DIR: &str = "data";

/// Generate a unique file path for the database file.
pub fn gen_db_file_path() -> String {
    let id = Uuid::new_v4();
    format!("{DATA_DIR}/test-{}.json", id)
}

/// Open a database and pass it to a closure.
/// The helper takes care of removing the database file after the closure is executed.
pub fn with_open_db<F>(f: F)
where
    F: FnOnce(Database),
{
    if !std::path::Path::new(DATA_DIR).exists() {
        std::fs::create_dir_all(DATA_DIR).unwrap();
    }

    let file_path = gen_db_file_path();
    let db = Database::open(&file_path).unwrap();
    f(db);
    std::fs::remove_file(file_path).unwrap();
}
