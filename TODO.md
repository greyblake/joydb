
- [x] Rename to `joydb`
- [x] Add description and keywords in Cargo.toml
- [x] Publish 0.0.1 on crates.io
- [ ] Introduce a flushing strategy. Would would be a good name for that? SyncMode!?

### Adapters

- [x] CSV adapter
- [ ] RON adapters
- [ ] YAML adapters

### Flushing
- [ ] Spawn a separate thread on `open` to flush database with some interval
  -  The thread should own a weak ref (see `Arc::downgrade`)


### CRUD methods:
- [ ] find_all_by(predicate) -> Vec<M>
- [ ] find_by(predicate) -> Option<M>
- [ ] delete_all_by(predicate) -> Vec<M>
- [ ] upsert


### Improvements
- [ ] Finalize naming of the methods
- [ ] Rename `GetRelation` to something else?
- [ ] Address all TODOs


### Setup CI
- [ ] On github


### Documentation
- [ ] Documentation for every public item
- [ ] Documentation in lib.rs
- [ ] Documentation in README
- [ ] Proper Cargo.toml: description, tags, keywords, etc.
- [ ] Come with a logo (see ideas https://chatgpt.com/c/67eab5c4-682c-800c-ad29-7d144b337bb9)

### Further ideas

- [ ] In memory only?
- [x] Rework `define_state!` macro to avoid need to specify plural names.
- [x] Reduce boilerplate in Db methods


- Rename?
    - (e.g. `playdb`, `loldb`, `jokedb`, `joydb`)
    - `alkali` (non-ACID :D )
    - `maydb`, `maybd`
    - `worm`, `storm`
    - `jorm` (Jörm - short from Jörmungandr) - also could be for "Json ORM".
    - `quasidb` (Quasi DB)
    - `shamdb` (Sham DB)
    - `bogusdb` (Bogus)
    - `notdb`, `keindb`
    - `undb`,
    - `fauxdb` - (Faux is French for "false" a short, catchy name that’s obviously not real.)
    - `neverdb`
    - `maldb` (like opposite from DB in Esperanto, or bad in Spanish)
    - `nondb`
    - `minidb`
    - `pseudb` (Pseudo DB)
    - `scratchdb`
    - `pocdb` - (Proof of Concept DB)
    - `pundb`
    - `nulldb`
    - `zerodb`
    - `filedb`
    - `sundb`


### Description

I am working on a Rust crate called `joydb`.
The idea of the crate is to:
- Be extremely easy to start using, avoiding all setup complexities of a real database and ORM
- Provide a persisting storage in files (like embedded DB)
- Keep the entire storage in memory (but flushing data to file system when necessary)
- Supporting multiple adapters (JSON, CSV, etc)
- Be very easy to use and simple by design
- Not intended for heavy load or serious production usage

