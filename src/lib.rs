use cppffi::{tree_ted, tree_topdiff_bounded};
use pgrx::prelude::*;

pgrx::pg_module_magic!();

mod lb;
mod parsing;
mod types;

use crate::lb::{
    label_intersection::{
        bounded_label_intersection_distance, inverted_bounded_lblint, inverted_lblint,
        label_intersection_distance,
    },
    sed::{
        bounded_sed, bounded_sed_opt, bounded_sed_opt_int, bounded_sed_struct,
        bounded_sed_struct_int, build_sed_indices_int, build_sed_struct_indices_int, sed,
    },
    structural_filter::ted as structural_lb,
    topdiff::ted_k,
};
use types::{tree_structural, InvertedTree};
use types::{
    SEDIndex, SEDStructIndex, StructuralFilter, StructuralSetConverter, TreeArena, UnifiedTreeIndex,
};

#[cxx::bridge]
mod cppffi {
    unsafe extern "C++" {
        include!("tree_similarity_extension/include/apted.h");
        include!("tree_similarity_extension/include/topdiff.h");

        fn tree_ted(a: String, b: String) -> u32;
        fn tree_topdiff_bounded(a: String, b: String, k: i32) -> u32;
    }
}

#[pg_extern]
fn add_node_to_tree_root(mut input_tree: TreeArena, node_value: String) -> TreeArena {
    if let Some(root_id) = input_tree.get_root_id() {
        let new_node = input_tree.new_node(node_value);
        root_id.append(new_node, &mut input_tree);
    } else {
        panic!("Tree has no root!")
    }

    input_tree
}

#[pg_extern(immutable, parallel_safe, cost = 1000)]
fn tree_lb_label_intersect(t1: TreeArena, t2: TreeArena) -> i32 {
    let lb = label_intersection_distance(&t1, &t2);
    lb as i32
}

#[pg_extern(immutable, parallel_safe, cost = 1000)]
fn tree_lb_bounded_label_intersect(t1: TreeArena, t2: TreeArena, lb: i32) -> i32 {
    let lb = bounded_label_intersection_distance(&t1, &t2, lb as usize);
    lb as i32
}

#[pg_extern(immutable, parallel_safe, cost = 1000)]
fn inverted_tree_label_intersect(t1: InvertedTree, t2: InvertedTree) -> i32 {
    inverted_lblint(&t1, &t2)
}

#[pg_extern(immutable, parallel_safe, cost = 1000)]
fn inverted_bounded_tree_label_intersect(t1: InvertedTree, t2: InvertedTree, lb: i32) -> i32 {
    // log!(
    //     "Running bounded LBL intersect between ts {} and {}",
    //     t1.tree_size,
    //     t2.tree_size
    // );
    inverted_bounded_lblint(&t1, &t2, lb as usize)
}

#[pg_extern(immutable, parallel_safe, cost = 1500)]
fn tree_lb_sed(t1: TreeArena, t2: TreeArena) -> i32 {
    let (t1, t2) = (SEDIndex::from(t1), SEDIndex::from(t2));
    let lb = sed(&t1, &t2);
    lb as i32
}

#[pg_extern(immutable, parallel_safe, cost = 1500)]
fn tree_lb_bounded_sed(t1: TreeArena, t2: TreeArena, lb: i32) -> i32 {
    let bound = lb as usize;

    let t_idx = std::time::Instant::now();
    let (t1, t2) = (SEDIndex::from(t1), SEDIndex::from(t2));
    let idx_ns = t_idx.elapsed().as_nanos();

    let t_sed = std::time::Instant::now();
    let result = bounded_sed(&t1, &t2, bound);
    let sed_ns = t_sed.elapsed().as_nanos();

    debug2!(
        "tree_lb_bounded_sed: sizes={}/{} bound={} result={} | SEDIndex::from={}ns bounded_sed={}ns total={}ns",
        t1.tree_size,
        t2.tree_size,
        bound,
        result,
        idx_ns,
        sed_ns,
        idx_ns + sed_ns
    );
    result as i32
}

#[pg_extern(immutable, parallel_safe, cost = 1000)]
fn sed_lb_sed(t1: SEDIndex, t2: SEDIndex) -> i32 {
    let lb = sed(&t1, &t2);
    lb as i32
}

#[pg_extern(immutable, parallel_safe, cost = 1000)]
fn sed_lb_bounded_sed(t1: SEDIndex, t2: SEDIndex, lb: i32) -> i32 {
    let lb = bounded_sed(&t1, &t2, lb as usize);
    lb as i32
}

// ---------------------------------------------------------------------------
// Optimized bounded SED (budget-constrained band)
// ---------------------------------------------------------------------------

#[pg_extern(immutable, parallel_safe, cost = 1500)]
fn tree_lb_bounded_sed_opt(t1: TreeArena, t2: TreeArena, lb: i32) -> i32 {
    let (t1, t2) = (SEDIndex::from(t1), SEDIndex::from(t2));
    bounded_sed_opt(&t1, &t2, lb as usize) as i32
}

#[pg_extern(immutable, parallel_safe, cost = 1000)]
fn sed_lb_bounded_sed_opt(t1: SEDIndex, t2: SEDIndex, lb: i32) -> i32 {
    bounded_sed_opt(&t1, &t2, lb as usize) as i32
}

// ---------------------------------------------------------------------------
// SED-STRUCT — bounded SED with structural pruning (sum/diff per node)
// ---------------------------------------------------------------------------

#[pg_extern(immutable, parallel_safe, cost = 2000)]
fn tree_lb_bounded_sed_struct(t1: TreeArena, t2: TreeArena, lb: i32) -> i32 {
    let (t1, t2) = (SEDStructIndex::from(t1), SEDStructIndex::from(t2));
    bounded_sed_struct(&t1, &t2, lb as usize) as i32
}

#[pg_extern(immutable, parallel_safe, cost = 1500)]
fn sed_struct_lb_bounded(t1: SEDStructIndex, t2: SEDStructIndex, lb: i32) -> i32 {
    bounded_sed_struct(&t1, &t2, lb as usize) as i32
}

#[pg_extern(immutable, parallel_safe, cost = 500)]
fn treearena_to_sed_struct_index(t1: TreeArena) -> SEDStructIndex {
    SEDStructIndex::from(t1)
}

// ---------------------------------------------------------------------------
// Integer-interned variants — build a local label→i32 dictionary from the two
// input trees, then compute bounded SED / SED-STRUCT on i32 slices. i32
// PartialEq is much cheaper than String::eq in the inner DP loop.
// ---------------------------------------------------------------------------

#[pg_extern(immutable, parallel_safe, cost = 1500)]
fn tree_lb_bounded_sed_int(t1: TreeArena, t2: TreeArena, lb: i32) -> i32 {
    let (i1, i2) = build_sed_indices_int(&t1, &t2);
    bounded_sed_opt_int(&i1, &i2, lb as usize) as i32
}

#[pg_extern(immutable, parallel_safe, cost = 2000)]
fn tree_lb_bounded_sed_struct_int(t1: TreeArena, t2: TreeArena, lb: i32) -> i32 {
    let (i1, i2) = build_sed_struct_indices_int(&t1, &t2);
    bounded_sed_struct_int(&i1, &i2, lb as usize) as i32
}

#[pg_extern(immutable, parallel_safe, cost = 1500)]
fn tree_lb_structural_filter(t1: TreeArena, t2: TreeArena, lb: i32) -> i32 {
    if t1.count().abs_diff(t2.count()) as i32 > lb {
        return lb + 1;
    }
    let mut lsc = StructuralSetConverter::default();
    let tree_tuples = lsc.create(&vec![t1, t2]);
    match &tree_tuples[..2] {
        [s1, s2] => structural_lb(s1, s2, lb),
        _ => panic!("Trees failed to convert!"),
    }
}

#[pg_extern(immutable, parallel_safe, cost = 1000)]
fn lb_structural_filter(t1: StructuralFilter, t2: StructuralFilter, lb: i32) -> i32 {
    structural_lb(&t1, &t2, lb)
}

#[pg_extern(immutable, parallel_safe, cost = 500)]
fn treearena_to_structural_filter_tuple(t1: TreeArena) -> StructuralFilter {
    let mut lsc = StructuralSetConverter::default();
    let mut tree_tuples = lsc.create(&vec![t1]);
    let Some(t) = tree_tuples.pop() else {
        panic!("Tree failed to convert")
    };
    t
}

#[pg_extern(immutable, parallel_safe, cost = 500)]
fn treearena_to_inverted_label_list(t1: TreeArena) -> InvertedTree {
    InvertedTree::from(t1)
}

#[pg_extern(immutable, parallel_safe, cost = 500)]
fn treearena_to_sed_index(t1: TreeArena) -> SEDIndex {
    SEDIndex::from(t1)
}

#[pg_extern(immutable, parallel_safe, cost = 5000)]
fn tree_ed(t1: TreeArena, t2: TreeArena) -> i32 {
    tree_ted(t1.to_string(), t2.to_string()) as i32
}

/// Bounded TopDiff (Touzet KR-set) tree edit distance. Returns the exact TED
/// when it is <= k, otherwise k+1 (over-bound), mirroring the other bounded LBs.
#[pg_extern(immutable, parallel_safe, cost = 5000)]
fn tree_topdiff_bounded_ed(t1: TreeArena, t2: TreeArena, k: i32) -> i32 {
    tree_topdiff_bounded(t1.to_string(), t2.to_string(), k) as i32
}

/// Combined SED-Struct LB filter → TopDiff verification pipeline over a single
/// pre-indexed `UnifiedTreeIndex` column. Each argument's CBOR is deserialized
/// once; both trees are interned into one shared dictionary, expanded into the
/// SED and TopDiff working forms, then run through Stage 1 (cheap SED-Struct
/// lower bound) and — only on survivors — Stage 2 (exact bounded TopDiff).
///
/// Returns the TopDiff distance when the pair passes both stages (`<= k`),
/// otherwise `k + 1` (over-bound), composable with `<= k` filters in SQL.
/// Sound: SED-Struct is a true lower bound on TED, so `LB > k ⇒ TED > k`.
#[pg_extern(immutable, parallel_safe, cost = 5000)]
fn sed_topdiff_within(query: UnifiedTreeIndex, cand: UnifiedTreeIndex, k: i32) -> i32 {
    if k < 0 {
        return k + 1;
    }
    let k_usize = k as usize;
    // Stage 0 — size-diff gate (also covers empty/oversized-diff pairs).
    if query.tree_size.abs_diff(cand.tree_size) > k_usize {
        return k + 1;
    }
    // Empty-tree fast path: TED(∅, T) = |T| (all inserts/deletes). `expand` and
    // `ted_k` are not defined for size-0 trees, so resolve here; the size-diff
    // gate above already returned k+1 when |T| exceeds k.
    if query.tree_size == 0 || cand.tree_size == 0 {
        return query.tree_size.max(cand.tree_size) as i32;
    }
    // One shared label dictionary across both trees and both working forms, so
    // Stage 1 and Stage 2 agree on which labels are equal.
    let mut dict = rustc_hash::FxHashMap::default();
    let (q_sed, q_td) = query.expand(&mut dict);
    let (c_sed, c_td) = cand.expand(&mut dict);
    // Stage 1 — SED-Struct lower bound (interned i32). Filtered out if LB > k.
    if bounded_sed_struct_int(&q_sed, &c_sed, k_usize) > k_usize {
        return k + 1;
    }
    // Stage 2 — exact bounded TopDiff; already returns k+1 on over-bound.
    ted_k(&q_td, &c_td, k)
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use crate::parsing::parse_tree;
    use crate::types::{TreeArena, UnifiedTreeIndex};
    use pgrx::prelude::*;
    use std::ffi::CString;

    /// A spread of small trees covering varied shapes (chains, bushy, deep),
    /// label overlap, and size differences.
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
    ];

    fn ta(s: &str) -> TreeArena {
        parse_tree(CString::new(s).unwrap().as_c_str()).unwrap()
    }
    fn uti(s: &str) -> UnifiedTreeIndex {
        UnifiedTreeIndex::from(ta(s))
    }

    /// Round-trip through the SQL boundary: bracket-notation input function +
    /// CBOR arg serialization + the full pipeline.
    #[pg_test]
    fn sql_round_trip() {
        let same = Spi::get_one::<i32>(
            "SELECT sed_topdiff_within('{a{b}{c}}'::unifiedtreeindex, '{a{b}{c}}'::unifiedtreeindex, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(same, 0);
        let one = Spi::get_one::<i32>(
            "SELECT sed_topdiff_within('{a{b}{c}}'::unifiedtreeindex, '{a{b}{x}}'::unifiedtreeindex, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(one, 1);
    }

    /// Acceptance gate: the whole pipeline must equal exact APTED TED, capped at
    /// k+1. This validates Stage-1 soundness (it never wrongly filters a within-k
    /// pair) AND Stage-2 exactness together. `tree_ed` is the independent APTED
    /// oracle.
    #[pg_test]
    fn within_matches_exact_ted() {
        for &a in TREES {
            for &b in TREES {
                let exact = crate::tree_ed(ta(a), ta(b));
                for &k in &[0i32, 1, 2, 3, 7, 50] {
                    let expected = if exact <= k { exact } else { k + 1 };
                    let got = crate::sed_topdiff_within(uti(a), uti(b), k);
                    assert_eq!(
                        got, expected,
                        "sed_topdiff_within({a}, {b}, {k}) = {got}, expected {expected} (exact TED {exact})"
                    );
                }
            }
        }
    }

    /// The pipeline must agree end-to-end with the retained C++ TopDiff oracle
    /// `tree_topdiff_bounded_ed` for every pair and k (both equal the capped TED).
    #[pg_test]
    fn pipeline_matches_cpp_oracle() {
        for &a in TREES {
            for &b in TREES {
                for &k in &[0i32, 1, 2, 3, 7, 50] {
                    let oracle = crate::tree_topdiff_bounded_ed(ta(a), ta(b), k);
                    let got = crate::sed_topdiff_within(uti(a), uti(b), k);
                    assert_eq!(
                        got, oracle,
                        "pipeline vs oracle mismatch for ({a}, {b}, k={k}): {got} != {oracle}"
                    );
                }
            }
        }
    }

    /// SED-Struct is a sound lower bound: its bounded LB never exceeds the exact
    /// TED. (If it did, Stage 1 could wrongly filter a true match.)
    #[pg_test]
    fn sed_struct_lb_is_sound() {
        for &a in TREES {
            for &b in TREES {
                let exact = crate::tree_ed(ta(a), ta(b));
                // Large bound so the LB is computed fully, not short-circuited.
                let lb = crate::tree_lb_bounded_sed_struct(ta(a), ta(b), 1000);
                assert!(
                    lb <= exact,
                    "SED-struct LB {lb} exceeds exact TED {exact} for ({a}, {b})"
                );
            }
        }
    }

    /// Edge cases: empty trees and negative k.
    #[pg_test]
    fn edge_cases() {
        let empty = || UnifiedTreeIndex {
            labels: vec![],
            sizes: vec![],
            tree_size: 0,
        };
        // Two empty trees: TED 0.
        assert_eq!(crate::sed_topdiff_within(empty(), empty(), 3), 0);
        // Empty vs 3-node tree within budget: TED 3.
        assert_eq!(crate::sed_topdiff_within(empty(), uti("{a{b}{c}}"), 5), 3);
        // Empty vs 3-node tree, budget too small: k+1.
        assert_eq!(crate::sed_topdiff_within(empty(), uti("{a{b}{c}}"), 1), 2);
        // Negative k is always over-bound (returns k+1).
        assert_eq!(crate::sed_topdiff_within(uti("{a}"), uti("{a}"), -1), 0);
    }
}

// ---------------------------------------------------------------------------
// Benchmarks (cargo pgrx bench) — gated behind the `pg_bench` feature.
//
// Compares the two SED-STRUCT bounded LB implementations head-to-head:
//   * tree_lb_bounded_sed_struct      — String-labelled SEDStructIndex path
//   * tree_lb_bounded_sed_struct_int  — i32-interned SEDStructIndexInt path
//
// The timed closure mirrors the body of each #[pg_extern] wrapper exactly
// (index build + bounded DP). Parsing the bracket string into a TreeArena is
// done in the *untimed* `iter_batched` setup — that step is the CBOR decode in
// real SQL and is identical for both, so excluding it isolates the difference.
// Each call gets a fresh TreeArena pair because `SEDStructIndex::from` consumes
// its input.
// ---------------------------------------------------------------------------
#[cfg(feature = "pg_bench")]
#[pg_schema]
mod benches {
    use pgrx::prelude::*;
    use pgrx_bench::{black_box, BatchSize, Bencher};

    use crate::lb::sed::{bounded_sed_struct, bounded_sed_struct_int, build_sed_struct_indices_int};
    use crate::parsing::parse_tree;
    use crate::types::{SEDStructIndex, TreeArena};
    use std::ffi::CString;

    // Two representative sentiment-treebank trees taken verbatim from trees.sql.
    const QUERY_TREE: &str = "{1{2 Something}{0{1{2 has}{1{2 been}{0{2 lost}{1{2 in}{0{2{2{2 the}{2 translation}}{2 ...}}{1{1{2 another}{2{2 routine}{1{2 Hollywood}{3 frightfest}}}}{1{2{2 in}{2 which}}{1{2{2 the}{1{2 slack}{2 execution}}}{2{2 italicizes}{1{1{2 the}{1 absurdity}}{2{2 of}{2{2 the}{2 premise}}}}}}}}}}}}}{2 .}}}";
    const DATA_TREE: &str = "{3{2{2 The}{1{2 dirty}{2 jokes}}}{3{4{2 provide}{3{3{2 the}{3{4 funniest}{2 moments}}}{3{2 in}{3{3{2 this}{3{3{2 oddly}{4 sweet}}{3 comedy}}}{2{2 about}{3{2 jokester}{2{2 highway}{2 patrolmen}}}}}}}}{2 .}}}";

    // Realistic edit-distance threshold (matches the dataset's thresholds, ~9-12).
    const K_REALISTIC: usize = 11;
    // Large threshold: defeats early short-circuits so the full DP runs and the
    // i32-vs-String inner-loop comparison cost dominates.
    const K_LARGE: usize = 500;

    /// Parse the embedded bracket strings into a fresh `TreeArena` pair.
    /// Runs in untimed setup, so its cost is excluded from the measurement.
    fn parse_pair() -> (TreeArena, TreeArena) {
        let q = CString::new(QUERY_TREE).unwrap();
        let d = CString::new(DATA_TREE).unwrap();
        (
            parse_tree(q.as_c_str()).unwrap(),
            parse_tree(d.as_c_str()).unwrap(),
        )
    }

    // --- realistic threshold ------------------------------------------------

    #[pg_bench]
    fn bench_sed_struct_realistic(b: &mut Bencher) {
        b.iter_batched(
            parse_pair,
            |(t1, t2)| {
                let (a, b) = (SEDStructIndex::from(t1), SEDStructIndex::from(t2));
                black_box(bounded_sed_struct(&a, &b, K_REALISTIC))
            },
            BatchSize::SmallInput,
        );
    }

    #[pg_bench]
    fn bench_sed_struct_int_realistic(b: &mut Bencher) {
        b.iter_batched(
            parse_pair,
            |(t1, t2)| {
                let (i1, i2) = build_sed_struct_indices_int(&t1, &t2);
                black_box(bounded_sed_struct_int(&i1, &i2, K_REALISTIC))
            },
            BatchSize::SmallInput,
        );
    }

    // --- large threshold (full DP) ------------------------------------------

    #[pg_bench]
    fn bench_sed_struct_large_k(b: &mut Bencher) {
        b.iter_batched(
            parse_pair,
            |(t1, t2)| {
                let (a, b) = (SEDStructIndex::from(t1), SEDStructIndex::from(t2));
                black_box(bounded_sed_struct(&a, &b, K_LARGE))
            },
            BatchSize::SmallInput,
        );
    }

    #[pg_bench]
    fn bench_sed_struct_int_large_k(b: &mut Bencher) {
        b.iter_batched(
            parse_pair,
            |(t1, t2)| {
                let (i1, i2) = build_sed_struct_indices_int(&t1, &t2);
                black_box(bounded_sed_struct_int(&i1, &i2, K_LARGE))
            },
            BatchSize::SmallInput,
        );
    }

    // --- full pipeline: unified vs separate-calls vs C++ oracle -------------
    //
    // The core hypothesis: a single combined function over one UnifiedTreeIndex
    // (one expand per argument) is faster than two separately-composed stages
    // and competitive with the C++ end-to-end TopDiff. NOTE: these micro-benches
    // exclude CBOR (de)serialization — the dominant SQL cost per CLAUDE.md. The
    // Approach-B "do String labels dominate the payload?" gate is best measured
    // on a loaded table via EXPLAIN ANALYZE, not in this harness.

    use crate::lb::topdiff::{ted_k, TopDiffIndex};
    use crate::types::UnifiedTreeIndex;

    const K_PIPE: i32 = K_REALISTIC as i32;

    /// Build a fresh `UnifiedTreeIndex` pair (untimed setup).
    fn unified_pair() -> (UnifiedTreeIndex, UnifiedTreeIndex) {
        let (t1, t2) = parse_pair();
        (UnifiedTreeIndex::from(t1), UnifiedTreeIndex::from(t2))
    }

    #[pg_bench]
    fn bench_unified_pipeline(b: &mut Bencher) {
        b.iter_batched(
            unified_pair,
            |(q, c)| black_box(crate::sed_topdiff_within(q, c, K_PIPE)),
            BatchSize::SmallInput,
        );
    }

    /// Baseline: the two stages built and run separately (no shared substrate /
    /// dict), mirroring two independently-composed `#[pg_extern]` calls.
    #[pg_bench]
    fn bench_separate_pipeline(b: &mut Bencher) {
        b.iter_batched(
            parse_pair,
            |(t1, t2)| {
                let (s1, s2) = build_sed_struct_indices_int(&t1, &t2);
                let lb = bounded_sed_struct_int(&s1, &s2, K_REALISTIC);
                let out = if lb > K_REALISTIC {
                    K_PIPE + 1
                } else {
                    let mut d1 = rustc_hash::FxHashMap::default();
                    let td1 = TopDiffIndex::from_tree(&t1, &mut d1);
                    let mut d2 = rustc_hash::FxHashMap::default();
                    let td2 = TopDiffIndex::from_tree(&t2, &mut d2);
                    ted_k(&td1, &td2, K_PIPE)
                };
                black_box(out)
            },
            BatchSize::SmallInput,
        );
    }

    #[pg_bench]
    fn bench_cpp_topdiff_oracle(b: &mut Bencher) {
        b.iter_batched(
            parse_pair,
            |(t1, t2)| {
                black_box(crate::tree_topdiff_bounded_ed(t1, t2, K_PIPE))
            },
            BatchSize::SmallInput,
        );
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
