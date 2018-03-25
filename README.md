# cdrs-helpers-derive
Procedural macros that derive helper traits for CDRS Cassandra to Rust types conversion back and forth

The package is under hard development and is absolutely not stable .

TODOs:

- [ ] convert Cassandra primitive types (not lists, sets, maps, UDTs) into Rust
- [ ] convert Cassandra "collection" types (lists, sets, maps) into Rust
- [ ] convert Cassandra UDTs into Rust
- [ ] convert optional fields into Rust
- [ ] convert Rust primitive types into Cassandra query values
- [ ] convert Rust "collection" types into Cassandra query values
- [ ] convert Rust structures into Cassandra query values
- [ ] convert `Option<T>` into Cassandra query value
