use crate::traits::{Adapter, RelationAdapter, State};
use std::path::Path;

struct JsonSingleFileAdapter;

impl<S: State> Adapter<S> for JsonSingleFileAdapter {
    fn read(&self, path: &Path) -> S {
        use std::io::Read;

        let mut file = std::fs::File::open(path).expect("Failed to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file");
        let plain_state: S::PlainState =
            serde_json::from_str(&contents).expect("Failed to deserialize JSON");
        S::from_plain(plain_state)
    }

    fn write(&self, path: &Path, state: S) {
        use std::io::Write;

        let json =
            serde_json::to_string_pretty(&state.to_plain()).expect("Failed to serialize JSON");
        let mut file = std::fs::File::create(path).expect("Failed to create file");
        file.write_all(json.as_bytes())
            .expect("Failed to write file");
    }
}
