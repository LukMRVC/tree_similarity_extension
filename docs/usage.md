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

## Building trees from data

Hierarchies stored as rows (adjacency lists and the like), JSON or XML can be turned into `tree` values.

All builders write labels the same way: a `\` goes before every `{` and `}` in a label, and stays part of the label (`a{b` becomes `a\{b`). A label ending in `\` would escape the brace after it, so it gets a trailing space; this is the one case where two different labels (`x\` and `x\ `) become equal.

### Adjacency lists: `tree_agg`

```sql
CREATE TABLE nodes (id int PRIMARY KEY, parent_id int REFERENCES nodes, label text, pos int);
INSERT INTO nodes VALUES (1, NULL, 'html', 1), (2, 1, 'head', 1), (3, 1, 'body', 2), (4, 3, 'p', 1);

SELECT tree_agg(id, parent_id, label ORDER BY pos) FROM nodes;   -- {html{head}{body{p}}}
```

`tree_agg(id, parent_id, label)` builds one tree from the rows of each group. Ids are `bigint` or `text`; cast other key types, e.g. `uuid`, with `::text`.

- **Root**: the one row whose `parent_id` is NULL or not the id of another row in the group. So a `WHERE` that keeps only the rows of a subtree gives that subtree.
- **Sibling order**: the order in which rows reach the aggregate, so give it an `ORDER BY`. Without one the order is unspecified, and TED depends on it.
- **Several trees**: one per group, e.g. `SELECT doc_id, tree_agg(...) FROM nodes GROUP BY doc_id`.
- **Errors**: no root or more than one root, a duplicate id, rows not connected to the root (their parent links form a cycle), a NULL id or label. No rows give NULL.

Other encodings of a hierarchy reduce to an adjacency list first:

| Encoding | `parent_id` of a row |
|----------|----------------------|
| Closure table `(ancestor, descendant, depth)` | `ancestor` of its row with `depth = 1` |
| Materialized path (`'1.4.2'`, or `ltree`) | its path without the last segment (`subpath(path, 0, -1)` for `ltree`) |
| Nested sets `(lft, rgt)` | the enclosing row with the largest `lft`: `(SELECT p.id FROM t p WHERE p.lft < c.lft AND p.rgt > c.rgt ORDER BY p.lft DESC LIMIT 1)`; order siblings by `lft` |

#### Without the extension: a recursive query

The same trees in plain SQL, one row per root. It walks each tree in preorder (siblings by `pos`, then `id`), opens a node per row, and after each row closes as many nodes as the depth drops to the next row:

<!-- recipe:adjacency -->
```sql
WITH RECURSIVE walk AS (
  SELECT id AS root, id, label, 1 AS depth, ARRAY[pos, id] AS path
  FROM nodes WHERE parent_id IS NULL
  UNION ALL
  SELECT w.root, n.id, n.label, w.depth + 1, w.path || ARRAY[n.pos, n.id]
  FROM nodes n JOIN walk w ON n.parent_id = w.id
)
SELECT root,
       string_agg('{' || regexp_replace(label, '([{}])', '\\\1', 'g')
                      || CASE WHEN right(label, 1) = '\' THEN ' ' ELSE '' END
                      || repeat('}', depth - coalesce(next_depth, 1) + 1),
                  '' ORDER BY path)::tree AS tree
FROM (SELECT *, lead(depth) OVER (PARTITION BY root ORDER BY path) AS next_depth FROM walk) s
GROUP BY root
```
<!-- /recipe:adjacency -->

It gives the same trees as `tree_agg(id, parent_id, label ORDER BY pos, id)`, except that rows not connected to a root are silently left out rather than reported.

### JSON: `tree_from_jsonb`

```sql
SELECT tree_from_jsonb('{"b": [1, "x"], "a": null}');   -- {\{\}{a{null}}{b{[]{1}{x}}}}
```

Follows the JSON tree model of JEDI (T. Hütter, N. Augsten et al., *JEDI: These aren't the JSON documents you're looking for…*, SIGMOD 2022):

- An object is a node `{}` (written `\{\}`). Its children are one node per key, sorted by key, since objects are unordered; each key node has the value as its only child.
- An array is a node `[]` with its elements in order.
- A string is a leaf with its text (no quotes); a number is a leaf with its text as Postgres prints it (`1.10` stays `1.10`); `true`, `false` and `null` are leaves with those labels.

For a `json` column use `tree_from_jsonb(doc::jsonb)`; `jsonb` keeps only the last of duplicate keys.

### XML: `tree_from_xml`

```sql
SELECT tree_from_xml('<article key="a/b" mdate="2012"><title>On <i>trees</i></title></article>');
-- {article{key{a/b}}{mdate{2012}}{title{On }{i{trees}}}}
```

The same layout as the XML-derived datasets (dblp, swissprot):

- An element is a node labelled with its name as written, prefix included (`x:tag`).
- Its attributes come first, sorted by name, each as `{name{value}}`. Namespace declarations (`xmlns`, `xmlns:x`) count as attributes.
- Then its content in document order: child elements, and text as leaves. Text is kept as written, surrounding whitespace included; text that is only whitespace is dropped. Adjacent text, CDATA and entity references form one leaf.
- Comments, processing instructions and the doctype are dropped.
- Predefined entities (`&lt;`, `&amp;`, …) and character references (`&#65;`) are decoded. Entities declared in a DTD (e.g. dblp's `&uuml;`) are kept as written.

The argument is `text`, because the `xml` type needs a Postgres built with libxml; for an `xml` column use `tree_from_xml(doc::text)`. The document must have exactly one root element.

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

A scan can check entries on several threads inside the backend. Results are identical at any thread count. The default is `4`; `1` checks entries on the backend alone.

```sql
SET tree_search_iam.scan_threads = 8;   -- 1 .. min(max(CPU count, 4), 64)
```

With `N > 1`, the backend reads the index and `N` worker threads check the entries while it reads ahead. The workers start with the first such scan and stay for the rest of the session, so later scans pay no startup cost.

The speedup depends on how much work each entry needs. With cheap checks the scan is limited by reading (rna: ~1.4× at 4–8 threads). With expensive ones it scales with threads (ptb: ~6× at 16). More threads than free CPU cores only slow the reading down.

This works for any plan that uses the index, including joins (one scan per outer row). It is separate from Postgres parallel query: each backend, including a parallel worker, keeps its own `N` threads.

### I/O method

For performance, use `io_method = worker` (the PG18 default). It is a server setting, so the extension cannot pick it per scan; changing it needs a restart:

```sql
ALTER SYSTEM SET io_method = 'worker';   -- then restart the server
```

It matters when the index is larger than a quarter of `shared_buffers`, so every scan re-reads it (see [Limitations](#limitations)). With `worker`, the I/O worker processes copy pages from the OS page cache on other cores while the backend checks entries. With `sync` and `io_uring`, a read that hits the page cache is copied by the backend itself. On python (523 MB index, `shared_buffers = 128MB`, warm page cache, 1 thread), a scan took ~135 ms with `worker` against ~197 ms with `io_uring` and ~199 ms with `sync`. `io_workers` above the default `3` gained nothing; `1` gave ~174 ms. An index that fits in shared buffers is not affected.

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
