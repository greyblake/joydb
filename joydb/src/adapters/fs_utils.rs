use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

/// Safely writes `content` to `target_path` using the write-rename pattern.
///
/// This function writes to a temporary file first, flushes it, then renames it atomically.
pub fn safe_write<P: AsRef<Path>>(target_path: P, content: &[u8]) -> io::Result<()> {
    let target_path = target_path.as_ref();
    let temp_path = target_path.with_extension("tmp");

    // Step 1: Write to temporary file
    let mut temp_file = File::create(&temp_path)?;
    temp_file.write_all(content)?;
    temp_file.sync_all()?; // Ensure it's flushed to disk

    // Step 2: Atomically rename temp file to target file
    fs::rename(&temp_path, target_path)?;

    Ok(())
}

/// Reads the content of a file and returns it as a `String`.
// Some of the adapters which are behind feature gate may not use this function.
#[allow(dead_code)]
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path.as_ref())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
