# Tree similarity PostgreSQL extension

### This is a WIP. Incompatible changes may and can occur at any time.

This project users [Pgrx](https://github.com/pgcentralfoundation/pgrx) framework to develop PostgreSQL extension
using [Rust](https://www.rust-lang.org/).

### Mentions

The storage type was heavily inspired by [indextree](https://github.com/saschagrunert/indextree/tree/main).
The API is not 1:1 the same was redone to play nicely with PostgreSQL as a custom type.

### Performance

Currently, custom type have a performance issue, since using Serialize and Deserialize from serde crate, all
`PostgresType` are serialized into [CBOR](https://cbor.io/) every time they are used. So every function call
has to deserialize this CBOR into memory representation. This introduces computational overhead when dealing
with big datasets. A better solution must be implemented.
