- [x] Rename to `joydb`
- [x] Add description and keywords in Cargo.toml
- [x] Publish 0.0.1 on crates.io
- [x] Introduce a flushing strategy. Would would be a good name for that? SyncMode!?
- [x] Add Justfile to run all the examples (as long as other tests and other checks)

### Adapters

- [x] Put adapters behind features
- [x] CSV adapter
- [x] RON adapters
- [x] Add examples for all the adapters.
- [x] Adjust adapters to write to a tmp file, than replace the original file. (Write-Rename pattern)

### Flushing
- [x] Spawn a separate thread on `open` to flush database with some interval. The thread should own a weak ref (see `Arc::downgrade`)


### CRUD methods:
- [x] Rename `find` to `get`
- [x] Rename `all` -> `get_all`
- [x] Implement `get_all_by()`
- [x] delete_all_by(predicate) -> Vec<M>
- [x] upsert(&M)


### Improvements
- [x] Finalize naming of the methods
- [x] Rename `GetRelation` to something else? (NO, leave it as it is)
- [ ] Address all TODOs
- [x] Refine error variants


### Setup CI
- [x] On github


### Documentation
- [ ] Documentation for every public item
- [ ] Documentation in lib.rs
- [ ] Documentation in README
- [x] Proper Cargo.toml: description, tags, keywords, etc.
- [x] Come with a logo

### Further ideas

- [x] In memory only?
- [x] Rework `define_state!` macro to avoid need to specify plural names.
- [x] Reduce boilerplate in Db methods


### Description

I am working on a Rust crate called `joydb`.
The idea of the crate is to:
- Be extremely easy to start using, avoiding all setup complexities of a real database and ORM
- Provide a persisting storage in files (like embedded DB)
- Keep the entire storage in memory (but flushing data to file system when necessary)
- Supporting multiple adapters (JSON, CSV, etc)
- Be very easy to use and simple by design
- Not intended for heavy load or serious production usage

