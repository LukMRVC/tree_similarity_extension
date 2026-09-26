# Tree similarity PostgreSQL extension

### This is a WIP. Incompatible changes may and can occur at any time.

PostgreSQL extension for tree similarity search: find rows whose tree edit distance (TED) to a query tree is `<= k`.
Built with [pgrx](https://github.com/pgcentralfoundation/pgrx) in [Rust](https://www.rust-lang.org/).

```sql
CREATE TABLE trees (id int PRIMARY KEY, tree tree);
INSERT INTO trees VALUES (1, '{a{b}{c}}'), (2, '{a{b}{x}}'), (3, '{x{y}{z}}');

CREATE INDEX ON trees USING tree_search_iam (tree) WITH (lb = sed_struct);

SELECT id FROM trees WHERE tree <~ tree_query('{a{b}{c}}', 1);   -- 1, 2
SELECT tree_ed('{a{b}{c}}', '{x{y}{z}}');                        -- 3
```

- `tree` type (bracket notation), `treequery` + `<~` operator
- `tree_search_iam` index access method with a selectable lower-bound filter
- Two-stage pipelines (lower bound → exact TopDiff) and standalone lower bounds
- Exact TED via APTED

**Full reference: [docs/usage.md](docs/usage.md).**

### Running locally

All development goes through `cargo pgrx`. The toolchain is pinned in `rust-toolchain.toml`, so `rustup` will
fetch the right `rustc` automatically.

```sh
# One-time setup: install the pgrx CLI (version-matched to the pinned pgrx dep) ...
cargo install --locked cargo-pgrx --version 0.18.0
# ... and download/build the supported Postgres versions into ~/.pgrx/.
cargo pgrx init

# Build + install the extension and drop into a psql session with it created.
# Defaults to pg18; pass pg17 to pick the other supported version (PG17+).
cargo pgrx run            # cargo pgrx run pg17

# Inside the psql session the extension is already loaded:
#   CREATE EXTENSION tree_similarity_extension;   -- (run automatically by `cargo pgrx run`)
#   SELECT tree_ed('{a{b}{c}}', '{a{b}{x}}');
#   SELECT '{a{b}}'::tree <~ tree_query('{a{c}}', 1);

# Re-open psql against the already-running instance without rebuilding.
cargo pgrx connect

# Run the tests inside a live Postgres backend (add a name filter or pgXX version).
cargo pgrx test

# Run the micro-benchmarks (gated behind the pg_bench feature).
cargo pgrx bench --features pg_bench

# Run the dataset search benchmark (pg18, datasets/ -> bench/results/results.csv).
./bench/run_bench.sh

# Start/stop the managed Postgres instances (ports 28800 + major version).
cargo pgrx start pg18
cargo pgrx stop  pg18
```

Pure-Rust unit/differential tests that don't need a Postgres backend (e.g. the TopDiff port in
`src/lb/ted/topdiff.rs`) also run under plain `cargo test`.

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

The Rust `ted_k` port in `src/lb/ted/topdiff.rs` is a faithful translation of tree-similarity's
`touzet_kr_set_tree_index_impl.cpp`, kept verified against the original C++ as a test oracle.

### Performance

`tree` stores only the bracket string, but it is still a serde `PostgresType`, so every function call decodes
[CBOR](https://cbor.io/) and re-parses the tree. The `tree_search_iam` index avoids this: trees are parsed once
at build time and stored as hashed labels + subtree sizes. The index is build-once (no WAL, goes stale on any
write until `REINDEX`) — see [docs/usage.md](docs/usage.md#limitations).
