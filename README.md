# Joydb

An in-memory embedded database with persistence and multiple adapters (JSON, CSV, etc).
Acts like a minimalistic ORM with zero setup.
Simple, lightweight, and perfect for prototypes, small apps, or experiments.


## Ideas

### Sync strategies

- None (manual)
- On every `write` operation
- In parallel thread (every N secods)

### Adapters

- JSON
- CSV
- RON
- YAML


### Similar projects

[lowdb](https://github.com/typicode/lowdb) - JSON database for JavaScript
[alkali](https://github.com/kneufeld/alkali) - Python ORM that writes to disk (JSON, YAML, CSV, etc)
