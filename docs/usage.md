# Usage

Similarity search over ordered, labelled trees: find all rows whose tree edit distance (TED) to a query tree is `<= k`.

```sql
CREATE EXTENSION tree_similarity_extension;
```

## Types

| Type        | What it is                                                        |
|-------------|-------------------------------------------------------------------|
| `tree`      | A tree in bracket notation. Input is validated; output is the input text. |
| `treequery` | A query tree plus a threshold `k`. Build it with `tree_query(tree, k)`. |

Bracket notation: `{label{child}{child}...}`. Labels are arbitrary strings (spaces allowed). A brace preceded by `\` is part of the label; the `\` is kept.

```sql
SELECT '{a{b}{c}}'::tree;
SELECT '{S{NP the cat}{VP sat}}'::tree;   -- labels with spaces
SELECT '{a{b\{c}}'::tree;                 -- label "b\{c"
SELECT 'x'::tree;                         -- ERROR: invalid tree "x": ...
```

## Similarity search: `<~`

`tree <~ tree_query(q, k)` is true iff `TED(tree, q) <= k`. It works with or without an index.

```sql
SELECT '{a{b}}'::tree <~ tree_query('{a{c}}', 1);   -- true
SELECT '{a{b}}'::tree <~ tree_query('{a{c}}', 0);   -- false

SELECT id FROM trees WHERE tree <~ tree_query('{a{b}{c}}', 2);
```

Query trees can come from another table (the index is rescanned per outer row):

```sql
SELECT q.id, t.id
FROM queries q JOIN trees t ON t.tree <~ tree_query(q.tree, q.k);
```

## Index: `tree_search_iam`

```sql
CREATE TABLE trees (id int PRIMARY KEY, tree tree);
-- ... load data ...
CREATE INDEX trees_idx ON trees USING tree_search_iam (tree);
CREATE INDEX trees_idx ON trees USING tree_search_iam (tree) WITH (lb = lblint);
```

Each tree is stored pre-parsed (hashed labels + subtree sizes). A scan checks every entry with the chosen lower bound, then verifies survivors with exact bounded TopDiff. Results are exact (no recheck) and returned as a bitmap scan:

```
Bitmap Heap Scan on trees
  ->  Bitmap Index Scan on trees_idx
        Index Cond: (tree <~ tree_query(...))
```

### `lb` option

Selects the Stage-1 lower-bound filter. Results are identical; only speed differs.

| `lb`                   | Filter                                |
|------------------------|---------------------------------------|
| `sed_struct` (default) | String edit distance + structural pruning |
| `sed_plain`            | Plain string edit distance (SED)      |
| `structural`           | Structural filter                     |
| `binary_branch`        | Binary branch distance                |
| `lblint`               | Label intersection                    |

### Scan threads

A scan can check entries on several threads inside the backend. Results are identical; the default `1` is single-threaded.

```sql
SET tree_search_iam.scan_threads = 8;   -- 1 .. min(CPU count, 64)
```

With `N > 1`, the backend reads the index and `N` worker threads check the entries while it reads ahead. The workers start with the first such scan and stay for the rest of the session, so later scans pay no startup cost.

The speedup depends on how much work each entry needs. With cheap checks the scan is limited by reading (rna: ~1.4× at 4–8 threads). With expensive ones it scales with threads (ptb: ~6× at 16). More threads than free CPU cores only slow the reading down.

This works for any plan that uses the index, including joins (one scan per outer row). It is separate from Postgres parallel query: each backend, including a parallel worker, keeps its own `N` threads.

### Limitations

- **Build-once.** Any `INSERT`/`UPDATE`, or a `VACUUM` that removes rows, marks the index stale. Scans then fail until rebuilt:
  ```
  ERROR: tree_search_iam index "trees_idx" is stale: rows were added or removed after it was built; run REINDEX INDEX trees_idx
  ```
  ```sql
  REINDEX INDEX trees_idx;
  ```
- No WAL logging (not crash-safe, not replicated). Unlogged tables are rejected.
- `NULL` trees are not indexed and never match.
- A scan reads every entry; the gain over a seq scan is skipping per-row parsing and the cheap LB filter.
- Pages are prefetched. An index larger than a quarter of `shared_buffers` is read through a small private buffer ring, like a large seq scan: it doesn't evict other data, but it is never cached in shared buffers, so every scan re-reads it (usually from the OS page cache).

The planner may still prefer a seq scan on small tables. To force the index:

```sql
SET enable_seqscan = off;
```

## Functions

### Pipelines (LB filter → exact TopDiff)

All return the exact TED when it is `<= k`, otherwise `k + 1`. Use as `f(q, t, k) <= k`.

| Function                                     | Stage-1 filter |
|----------------------------------------------|----------------|
| `sed_topdiff_within(q tree, t tree, k int)`           | SED-Struct |
| `sed_plain_topdiff_within(q tree, t tree, k int)`     | SED        |
| `structural_topdiff_within(q tree, t tree, k int)`    | Structural |
| `binary_branch_topdiff_within(q tree, t tree, k int)` | Binary branch |
| `lblint_topdiff_within(q tree, t tree, k int)`        | Label intersection |

```sql
SELECT sed_topdiff_within('{a{b}{c}}', '{a{b}{x}}', 5);   -- 1
SELECT id FROM trees WHERE sed_topdiff_within('{a{b}{c}}', tree, 2) <= 2;
```

### Exact distance

| Function                                    | Returns |
|---------------------------------------------|---------|
| `tree_ed(a tree, b tree)`                   | Exact TED (APTED, C++) |
| `tree_topdiff_bounded_ed(a tree, b tree, k int)` | Exact TED if `<= k`, else `k + 1` (TopDiff, C++) |

```sql
SELECT tree_ed('{a{b}{c}}', '{a{b}{x}}');   -- 1
```

### Standalone lower bounds

Return a lower bound on TED. Bounded variants (`lb` = threshold) may stop early and return any value `> lb`.

| Function | Bound |
|----------|-------|
| `tree_lb_sed(a, b)`, `tree_lb_bounded_sed(a, b, lb)`, `tree_lb_bounded_sed_opt(a, b, lb)`, `tree_lb_bounded_sed_int(a, b, lb)` | SED |
| `tree_lb_bounded_sed_struct(a, b, lb)`, `tree_lb_bounded_sed_struct_int(a, b, lb)` | SED-Struct |
| `tree_lb_structural_filter(a, b, lb)` | Structural |
| `tree_lb_label_intersect(a, b)`, `tree_lb_bounded_label_intersect(a, b, lb)` | Label intersection |

`*_int` variants compare hashed labels instead of strings.

### Precomputed index types

Convert once, store in a column, compare without re-parsing:

| Convert                              | Type               | Compare |
|--------------------------------------|--------------------|---------|
| `tree_to_sed_index(tree)`            | `sedindex`         | `sed_lb_sed`, `sed_lb_bounded_sed`, `sed_lb_bounded_sed_opt` |
| `tree_to_sed_struct_index(tree)`     | `sedstructindex`   | `sed_struct_lb_bounded` |
| `tree_to_structural_filter_tuple(tree)` | `structuralfilter` | `lb_structural_filter` |
| `tree_to_inverted_label_list(tree)`  | `invertedtree`     | `inverted_tree_label_intersect`, `inverted_bounded_tree_label_intersect` |

```sql
ALTER TABLE trees ADD COLUMN sed sedstructindex;
UPDATE trees SET sed = tree_to_sed_struct_index(tree);
SELECT id FROM trees
WHERE sed_struct_lb_bounded(tree_to_sed_struct_index('{a{b}{c}}'), sed, 2) <= 2;
```

### Misc

- `add_node_to_tree_root(t tree, label text) → tree` — appends a leaf under the root.

## Benchmarks

```sh
./bench/run_bench.sh                          # build, load datasets/, run all
./bench/run_bench.sh --skip-build --only-dataset rna --only-method iam_lblint
./bench/run_bench.sh --load-only
./bench/run_bench.sh --skip-build --only-method iam_lblint@t8   # index scan on 8 threads
```

Runs 100 queries per (dataset, method); methods are the five `*_topdiff_within` functions and `iam_<lb>` (index built `WITH (lb = <lb>)`; add `@t<N>` for `scan_threads = N`). Resumable; results go to `bench/results/results.csv`. Needs `bracket-prep` from `../ted-search`.
