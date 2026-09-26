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
cargo install --locked cargo-pgrx --version 0.19.1
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

### I/O configuration and io_uring

Index scans read pages through a Postgres read stream, which prefetches ahead of the scan. How those reads are
executed is Postgres's `io_method` (PG18). These settings affect scan speed:

| Setting                        | Default        | Effect                                                                                                                    |
|--------------------------------|----------------|---------------------------------------------------------------------------------------------------------------------------|
| `io_method`                    | `worker`       | `worker` (I/O worker processes), `sync` (backend reads itself), `io_uring` (Linux, needs a `--with-liburing` build). Restart. |
| `io_workers`                   | `3`            | I/O worker processes; `worker` only.                                                                                      |
| `effective_io_concurrency`     | `16`           | How many reads a stream keeps in flight.                                                                                  |
| `io_combine_limit`             | `128kB`        | Adjacent pages merged into one read (capped by `io_max_combine_limit`, restart).                                          |
| `shared_buffers`               | `128MB`        | An index larger than 1/4 of it is read through a small private ring: it evicts nothing else, but every scan re-reads it. |
| `tree_search_iam.scan_threads` | `1`            | Threads checking entries per scan (per session).                                                                          |

On WSL2, with the index not in shared buffers, `worker` was ~2× slower than `sync` for index scans (rna, 20
queries: ~590 ms vs ~200 ms), and Postgres's own seq scan was ~5× slower. `io_uring` is untested so far.

#### Benchmarking on an io_uring system

`cargo pgrx init` always builds an assertion-enabled Postgres (`--enable-cassert`, `USE_ASSERT_CHECKING`,
`RANDOMIZE_ALLOCATED_MEMORY`), which is fine for tests but skews timings, and it has no liburing support. For
performance numbers, use a release Postgres 18 built with liburing:

```sh
# 1. The kernel must allow io_uring (0 = allowed). Container seccomp profiles may block it.
cat /proc/sys/kernel/io_uring_disabled
# 2. liburing headers, plus Postgres's usual build dependencies.
sudo apt install liburing-dev            # Fedora: sudo dnf install liburing-devel
# 3. Build Postgres 18 from source (meson: -Dliburing=enabled).
./configure --prefix=$HOME/pg18-uring --with-liburing && make -j"$(nproc)" && make install
$HOME/pg18-uring/bin/pg_config --configure | grep -o -- --with-liburing   # packaged builds: check the same way
# 4. Cluster with io_uring, on its own port.
$HOME/pg18-uring/bin/initdb -D $HOME/pg18-uring/data
echo "io_method = io_uring" >> $HOME/pg18-uring/data/postgresql.conf
$HOME/pg18-uring/bin/pg_ctl -D $HOME/pg18-uring/data -o "-p 5418" -l $HOME/pg18-uring/log start
$HOME/pg18-uring/bin/psql -p 5418 -d postgres -c "SHOW io_method"                 # io_uring
# 5. Point pgrx at it and run the benchmark into a separate results directory.
cargo pgrx init --pg18 $HOME/pg18-uring/bin/pg_config
PG_CONFIG=$HOME/pg18-uring/bin/pg_config PGPORT=5418 OUT_DIR=bench/results/io_uring ./bench/run_bench.sh
```

To compare methods on the same server, switch and restart between runs, each into its own `OUT_DIR`:
`results.csv` is resumable by (dataset, method, query), so a shared file would skip every query as already done.

```sh
psql -p 5418 -d postgres -c "ALTER SYSTEM SET io_method = 'worker'"   # or 'sync'; RESET for the default
pg_ctl -D $HOME/pg18-uring/data restart
PG_CONFIG=... PGPORT=5418 OUT_DIR=bench/results/worker ./bench/run_bench.sh --skip-build
```

I/O only matters when the index is not already in shared buffers: use an index larger than `shared_buffers`
(treefam's is 153 MB) or lower `shared_buffers`.
