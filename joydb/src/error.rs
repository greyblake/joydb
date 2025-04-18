use std::path::PathBuf;
use thiserror::Error;

/// Joydb error type.
#[derive(Debug, Error)]
pub enum JoydbError {
    /// File system related error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// A path expected to be a file, but it is not.
    #[error("{0} is not a file")]
    NotFile(PathBuf),

    /// A path expected to be a directory, but it is not.
    #[error("{0} is not a directory")]
    NotDirectory(PathBuf),

    /// Serialization error.
    /// This may occur when adapter format is not supporting the data type.
    /// For example, if you try to serialize a HashMap with K type as a complex structure to JSON.
    #[error("Serialize error: {0}")]
    Serialize(Box<dyn std::error::Error + Send + Sync>),

    /// Deserialization error.
    /// May occur on opening a file.
    #[error("Deserialize error: {0}")]
    Deserialize(Box<dyn std::error::Error + Send + Sync>),

    /// Error when trying to insert a model with an ID that already exists.
    #[error("{model} with id = {id} already exists")]
    DuplicatedId {
        /// ID of the model formatted with `Debug`
        id: String,
        /// Name of the model (type name)
        model: String,
    },

    /// Error when trying to update a model with an ID that does not exist.
    #[error("{model} with id = {id} not found")]
    NotFound {
        /// ID of the model formatted with `Debug`
        id: String,
        /// Name of the model (type name)
        model: String,
    },

    /// Custom error variant. Intended for third party adapters for situations
    /// when non of the existing variants are suitable.
    #[error("Custom error: {0}")]
    Custom(Box<dyn std::error::Error + Send + Sync>),
}
