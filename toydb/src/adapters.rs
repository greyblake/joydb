use crate::{
    ToydbError,
    traits::{Adapter, State},
};
use std::io::{Read, Write};
use std::path::Path;

pub struct JsonAdapter;

impl Adapter for JsonAdapter {
    fn read<S: State>(path: &Path) -> Result<S, ToydbError> {
        let mut file = std::fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let state = serde_json::from_str(&contents)?;
        Ok(state)
    }

    fn write<S: State>(path: &Path, state: &S) -> Result<(), ToydbError> {
        let json = serde_json::to_string_pretty(state)?;
        let mut file = std::fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
