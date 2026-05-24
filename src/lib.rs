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
};
use types::{tree_structural, InvertedTree};
use types::{SEDIndex, SEDStructIndex, StructuralFilter, StructuralSetConverter, TreeArena};

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

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    // TODO: Add postgres tests
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
