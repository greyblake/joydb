use std::{fmt, path::PathBuf};

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    NotFile(PathBuf),
    Json(serde_json::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Io(err) => write!(f, "IO error: {}", err),
            StorageError::NotFile(path) => write!(f, "{} is not a file", path.display()),
            StorageError::Json(err) => write!(f, "JSON error: {}", err),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(err) => Some(err),
            StorageError::Json(err) => Some(err),
            StorageError::NotFile(_) => None,
        }
    }
}

// Implement From conversions
impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::Json(err)
    }
}
