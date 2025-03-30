#[macro_export]
macro_rules! define_storage {
    (
        $storage_name:ident,
        $(
            $plural:ident : $record_type:ty
        ),* $(,)?
    ) => {
        // A struct that represents the inner state of the storage.
        // TODO: Ideally it should be constructed dynamically like `${storage_name}State`, but it
        // would require extra dependencies like `paste`.
        #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
        #[serde(default)]
        struct StorageState {
            $(
                $plural: Vec<$record_type>
            ),+
        }

        /// File-based JSON storage that mimics a database and provides a simple interface
        /// to fetch and add information.
        pub struct $storage_name {
            state: StorageState,
            file_path: ::std::path::PathBuf,
        }

        // Define the relation methods on Storage
        //
        impl $storage_name {
            /// Try to load a database from the give file path.
            /// If file does not exist yet, then create a new one.
            pub fn open(file_path: impl Into<::std::path::PathBuf>) -> Result<Self, ::toydb::StorageError> {
                let file_path = file_path.into();
                let path = ::std::path::Path::new(&file_path);
                if path.exists() {
                    if path.is_file() {
                        Self::load(file_path)
                    } else {
                        Err(::toydb::StorageError::NotFile(file_path))
                    }
                } else {
                    Self::create(file_path)
                }
            }

            pub fn flush(&self) -> Result<(), ::toydb::StorageError> {
                let content = ::serde_json::to_string_pretty(&self.state)?;
                ::std::fs::write(&self.file_path, content)?;
                Ok(())
            }

            fn create(file_path: impl Into<::std::path::PathBuf>) -> Result<Self, ::toydb::StorageError> {
                let db = Self::new(file_path);
                db.flush()?;
                Ok(db)
            }

            fn new(file_path: impl Into<::std::path::PathBuf>) -> Self {
                Self {
                    state: StorageState::default(),
                    file_path: file_path.into(),
                }
            }

            fn load(file_path: impl Into<::std::path::PathBuf>) -> Result<Self, ::toydb::StorageError> {
                let file_path = file_path.into();
                let content = std::fs::read_to_string(&file_path)?;
                let state: StorageState = serde_json::from_str(&content)?;
                Ok(Self { file_path, state })
            }

            // Define relations
            $(
                pub fn $plural(&mut self) -> ::toydb::Relation<$record_type> {
                    ::toydb::Relation::new(&mut self.state.$plural)
                }
            )+
        }

        impl ::std::ops::Drop for $storage_name {
            fn drop(&mut self) {
                if let Err(err) = self.flush() {
                    eprintln!("Failed to flush the database: {}", err);
                }
            }
        }
    }
}
