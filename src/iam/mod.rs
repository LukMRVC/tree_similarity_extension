//! `tree_search_iam`: an index access method over a `tree` column that answers
//! `tree <~ tree_query(q, k)` (TED <= k).
//!
//! Every tree is stored as a `UnifiedTreeIndex` substrate. A scan reads all
//! entries, filters them with the index's lower bound (`WITH (lb = ...)`) and
//! verifies survivors with exact bounded TopDiff, so results are exact and
//! returned as a TID bitmap with no recheck.
//!
//! No WAL and no incremental maintenance: any insert, update or VACUUM that
//! removes rows marks the index stale, and scans then error until REINDEX.

mod build;
mod cost;
mod insert;
pub mod options;
mod parallel;
mod scan;
mod storage;

use pgrx::prelude::*;

/// Register the index's reloptions and GUCs. Runs once per backend, from `_PG_init`.
pub fn register() {
    options::register();
    scan::register_guc();
}

#[pg_extern(sql = "
    CREATE FUNCTION tree_search_iam_handler(internal) RETURNS index_am_handler
        PARALLEL SAFE IMMUTABLE STRICT LANGUAGE c AS 'MODULE_PATHNAME', '@FUNCTION_NAME@';
    CREATE ACCESS METHOD tree_search_iam TYPE INDEX HANDLER tree_search_iam_handler;
")]
fn tree_search_iam_handler(_fcinfo: pg_sys::FunctionCallInfo) -> PgBox<pg_sys::IndexAmRoutine> {
    // alloc_node zeroes the struct, so version-specific fields default to off.
    let mut am = unsafe { PgBox::<pg_sys::IndexAmRoutine>::alloc_node(pg_sys::NodeTag::T_IndexAmRoutine) };

    am.amstrategies = 1;
    am.amsupport = 0;
    am.amoptionalkey = false;
    am.amsearchnulls = false;
    am.amkeytype = pg_sys::InvalidOid;

    am.ambuild = Some(build::ambuild);
    am.ambuildempty = Some(build::ambuildempty);
    am.aminsert = Some(insert::aminsert);
    am.ambulkdelete = Some(insert::ambulkdelete);
    am.amvacuumcleanup = Some(insert::amvacuumcleanup);
    am.amcostestimate = Some(cost::amcostestimate);
    am.amoptions = Some(options::amoptions);
    am.amvalidate = Some(amvalidate);
    am.ambeginscan = Some(scan::ambeginscan);
    am.amrescan = Some(scan::amrescan);
    am.amgetbitmap = Some(scan::amgetbitmap);
    am.amendscan = Some(scan::amendscan);

    am.into_pg_boxed()
}

#[pg_guard]
unsafe extern "C-unwind" fn amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    true
}

extension_sql!(
    "CREATE OPERATOR CLASS tree_search_iam_ops DEFAULT FOR TYPE tree USING tree_search_iam AS
        OPERATOR 1 <~ (tree, treequery);",
    name = "tree_search_iam_ops",
    requires = [
        tree_search_iam_handler,
        types::tree::tree_within,
    ]
);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    use crate::pipelines::Lb;

    const TREES: &[&str] = &[
        "{a}",
        "{a{b}}",
        "{a{b}{c}}",
        "{a{b}{c}{d}}",
        "{a{b{e}}{c}}",
        "{x{y}{z}}",
        "{a{b}{x}}",
        "{r{a{b}{c}}{d{e}}}",
        "{r{a{b}{c}}{d{f}}}",
        "{1{2}{3{4}}}",
        "{a b{c d}{e\\{f}}",
    ];

    fn create_table(name: &str) {
        Spi::run(&format!("CREATE TABLE {name} (id int PRIMARY KEY, tree tree)")).unwrap();
        for (i, t) in TREES.iter().enumerate() {
            let t = t.replace('\'', "''");
            Spi::run(&format!("INSERT INTO {name} VALUES ({i}, '{t}')")).unwrap();
        }
    }

    fn ids(query: &str) -> Vec<i32> {
        Spi::get_one::<Vec<i32>>(&format!(
            "SELECT coalesce(array_agg(id ORDER BY id), '{{}}') FROM ({query}) s"
        ))
        .unwrap()
        .unwrap()
    }

    fn explain(query: &str) -> String {
        Spi::connect(|client| {
            client
                .select(&format!("EXPLAIN (COSTS OFF) {query}"), None, &[])
                .unwrap()
                .map(|row| row.get::<String>(1).unwrap().unwrap())
                .collect::<Vec<_>>()
                .join("\n")
        })
    }

    fn force_index() {
        Spi::run("SET LOCAL enable_seqscan = off").unwrap();
    }

    fn scan_threads(n: usize) {
        Spi::run(&format!("SET LOCAL tree_search_iam.scan_threads = {n}")).unwrap();
    }

    /// Thread counts the scan tests run with: inline, and on spawned threads.
    const THREADS: [usize; 2] = [1, 4];

    /// Rows with TED <= k by the C++ oracle, over a plain seq scan.
    fn expected(table: &str, q: &str, k: i32) -> Vec<i32> {
        ids(&format!(
            "SELECT id FROM {table} WHERE tree_topdiff_bounded_ed(tree, '{q}'::tree, {k}) <= {k}"
        ))
    }

    #[pg_test]
    fn tree_text_round_trip() {
        let out = Spi::get_one::<String>("SELECT '{a{b}{c\\}d}}'::tree::text").unwrap().unwrap();
        assert_eq!(out, "{a{b}{c\\}d}}");
    }

    #[pg_test(error = "invalid tree \"x\": tree string has incorrect bracket notation format: Tree has no root node")]
    fn tree_rejects_bad_input() {
        Spi::run("SELECT 'x'::tree").unwrap();
    }

    #[pg_test]
    fn operator_without_index() {
        let yes = Spi::get_one::<bool>("SELECT '{a{b}}'::tree <~ tree_query('{a{c}}', 1)").unwrap();
        let no = Spi::get_one::<bool>("SELECT '{a{b}}'::tree <~ tree_query('{a{c}}', 0)").unwrap();
        assert_eq!((yes, no), (Some(true), Some(false)));
    }

    /// For every `lb`, the index returns exactly the rows the oracle does, and
    /// the planner answers through a bitmap index scan.
    #[pg_test]
    fn index_matches_oracle_for_every_lb() {
        create_table("t_all");
        Spi::run("INSERT INTO t_all VALUES (100, NULL)").unwrap();
        for lb in Lb::ALL {
            let name = lb.name();
            Spi::run(&format!(
                "CREATE INDEX t_all_idx ON t_all USING tree_search_iam (tree) WITH (lb = {name})"
            ))
            .unwrap();
            force_index();
            let plan = explain("SELECT id FROM t_all WHERE tree <~ tree_query('{a}', 1)");
            assert!(plan.contains("Bitmap Index Scan on t_all_idx"), "index not used for lb={name}:\n{plan}");
            for threads in THREADS {
                scan_threads(threads);
                for q in TREES {
                    let q = q.replace('\'', "''");
                    for k in [0, 1, 2, 3, 5] {
                        let query = format!("SELECT id FROM t_all WHERE tree <~ tree_query('{q}', {k})");
                        let msg = format!("lb={name} q={q} k={k} threads={threads}");
                        assert_eq!(ids(&query), expected("t_all", &q, k), "{msg}");
                    }
                }
            }
            Spi::run("RESET enable_seqscan").unwrap();
            Spi::run("DROP INDEX t_all_idx").unwrap();
        }
    }

    /// The bench shape: the query tree comes from another table, so the index
    /// is rescanned with a new key for every outer row.
    #[pg_test]
    fn index_join_rescans() {
        create_table("t_join");
        Spi::run("CREATE TABLE q_join (id int PRIMARY KEY, k int, tree tree)").unwrap();
        Spi::run("INSERT INTO q_join SELECT id, id % 4, tree FROM t_join").unwrap();
        Spi::run("CREATE INDEX t_join_idx ON t_join USING tree_search_iam (tree)").unwrap();
        force_index();
        let want = Spi::get_one::<i64>(
            "SELECT count(*) FROM q_join q, t_join t
             WHERE tree_topdiff_bounded_ed(t.tree, q.tree, q.k) <= q.k",
        )
        .unwrap()
        .unwrap();
        for threads in THREADS {
            scan_threads(threads);
            let got = Spi::get_one::<i64>(
                "SELECT count(*) FROM q_join q, t_join t WHERE t.tree <~ tree_query(q.tree, q.k)",
            )
            .unwrap()
            .unwrap();
            assert_eq!(got, want, "threads={threads}");
        }
    }

    #[pg_test]
    fn empty_table() {
        Spi::run("CREATE TABLE t_empty (id int, tree tree)").unwrap();
        Spi::run("CREATE INDEX t_empty_idx ON t_empty USING tree_search_iam (tree)").unwrap();
        force_index();
        assert!(ids("SELECT id FROM t_empty WHERE tree <~ tree_query('{a}', 3)").is_empty());
    }

    /// Trees far bigger than one index page, mixed with small ones.
    #[pg_test]
    fn multi_page_trees() {
        Spi::run("CREATE TABLE t_big (id int PRIMARY KEY, tree tree)").unwrap();
        Spi::run(
            "INSERT INTO t_big
             SELECT n, (repeat('{x', n * 50) || repeat('}', n * 50))::tree FROM generate_series(1, 30) n",
        )
        .unwrap();
        Spi::run("CREATE INDEX t_big_idx ON t_big USING tree_search_iam (tree)").unwrap();
        force_index();
        let q = "(repeat('{x', 500) || repeat('}', 500))::tree";
        for threads in THREADS {
            scan_threads(threads);
            assert_eq!(ids(&format!("SELECT id FROM t_big WHERE tree <~ tree_query({q}, 0)")), vec![10]);
            assert_eq!(
                ids(&format!("SELECT id FROM t_big WHERE tree <~ tree_query({q}, 50)")),
                vec![9, 10, 11]
            );
        }
    }

    #[pg_test(
        error = "tree_search_iam index \"t_stale_idx\" is stale: rows were added or removed after it was built; run REINDEX INDEX t_stale_idx"
    )]
    fn insert_makes_index_stale() {
        create_table("t_stale");
        Spi::run("CREATE INDEX t_stale_idx ON t_stale USING tree_search_iam (tree)").unwrap();
        Spi::run("INSERT INTO t_stale VALUES (1000, '{a}')").unwrap();
        force_index();
        ids("SELECT id FROM t_stale WHERE tree <~ tree_query('{a}', 0)");
    }

    #[pg_test]
    fn reindex_picks_up_new_rows() {
        create_table("t_re");
        Spi::run("CREATE INDEX t_re_idx ON t_re USING tree_search_iam (tree)").unwrap();
        Spi::run("INSERT INTO t_re VALUES (1000, '{a}')").unwrap();
        Spi::run("REINDEX INDEX t_re_idx").unwrap();
        force_index();
        assert_eq!(ids("SELECT id FROM t_re WHERE tree <~ tree_query('{a}', 0)"), vec![0, 1000]);
    }

    /// `SET` on an unregistered name makes a string placeholder, so the thread
    /// tests would silently run serially if registration broke.
    #[pg_test]
    fn scan_threads_is_registered() {
        let row = Spi::get_one::<String>(
            "SELECT vartype || ' ' || min_val FROM pg_settings WHERE name = 'tree_search_iam.scan_threads'",
        )
        .unwrap();
        assert_eq!(row.as_deref(), Some("integer 1"));
    }

    #[pg_test(error = "invalid value for enum option \"lb\": nope")]
    fn bad_lb_rejected() {
        create_table("t_bad");
        Spi::run("CREATE INDEX t_bad_idx ON t_bad USING tree_search_iam (tree) WITH (lb = nope)").unwrap();
    }

    #[pg_test(error = "tree_search_iam does not support unlogged tables")]
    fn unlogged_rejected() {
        Spi::run("CREATE UNLOGGED TABLE t_unlogged (tree tree)").unwrap();
        Spi::run("CREATE INDEX ON t_unlogged USING tree_search_iam (tree)").unwrap();
    }
}
