use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToydbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0} is not a file")]
    NotFile(PathBuf),

    #[error("{0} is not a directory")]
    NotDirectory(PathBuf),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{model_name} with id = {id} already exists")]
    DuplicatedId { id: String, model_name: String },

    #[error("{model_name} with id = {id} not found")]
    NotFound { id: String, model_name: String },
}
