# Tree similarity PostgreSQL extension

### This is a WIP. Incompatible changes may and can occur at any time.

This project users [Pgrx](https://github.com/pgcentralfoundation/pgrx) framework to develop PostgreSQL extension
using [Rust](https://www.rust-lang.org/).

### Mentions

The storage type was heavily inspired by [indextree](https://github.com/saschagrunert/indextree/tree/main).
The API is not 1:1 the same was redone to play nicely with PostgreSQL as a custom type.

### Algorithms & attribution

The tree-edit-distance algorithms in this extension originate from the
[**tree-similarity**](https://github.com/DatabaseGroup/tree-similarity/) library by the
Database Research Group, University of Salzburg (MIT licensed). The C++ sources under
`src_cpp/` and `include/` are vendored from that project and retain their original copyright
headers (© 2017–2019 Mateusz Pawlik, Nikolaus Augsten, Daniel Kocher, Thomas Huetter).
Full credit for the algorithms and their reference implementations goes to those authors.

- **APTED** — the exact tree edit distance (`tree_ed`) — by **Mateusz Pawlik** and **Nikolaus Augsten**:
  - M. Pawlik and N. Augsten. *RTED: A Robust Algorithm for the Tree Edit Distance.* PVLDB, 2011.
  - M. Pawlik and N. Augsten. *A Memory-Efficient Tree Edit Distance Algorithm.* DEXA, 2014.
  - M. Pawlik and N. Augsten. *Efficient Computation of the Tree Edit Distance.* ACM TODS, 2015.
  - M. Pawlik and N. Augsten. *Tree edit distance: Robust and memory-efficient.* Information Systems, 2016.
- **TopDiff** (the `TouzetKRSetTreeIndex` keyroot-set bounded TED — `tree_topdiff_bounded_ed`, and the Rust
  port driving `sed_topdiff_within`) — based on Hélène Touzet's algorithm, as implemented in tree-similarity:
  - H. Touzet. *Comparing similar ordered trees in linear time.* Journal of Discrete Algorithms, 2007.

The Rust `ted_k` port in `src/lb/topdiff.rs` is a faithful translation of tree-similarity's
`touzet_kr_set_tree_index_impl.cpp`, kept verified against the original C++ as a test oracle.

### Performance

Currently, custom type have a performance issue, since using Serialize and Deserialize from serde crate, all
`PostgresType` are serialized into [CBOR](https://cbor.io/) every time they are used. So every function call
has to deserialize this CBOR into memory representation. This introduces computational overhead when dealing
with big datasets. A better solution must be implemented.
