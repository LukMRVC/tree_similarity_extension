//! Plain bounded-SED LB filter → TopDiff verification pipeline.
//!
//! Mirror of `sed_topdiff_within` (lib.rs) but with a *plain* Stage-1 filter:
//! the string edit distance between the two trees' label-only traversal
//! sequences (preorder + postorder), WITHOUT the SED-Struct `sum`/`diff`
//! annotations. Plain SED on the traversals is a true lower bound on the tree
//! edit distance, so `LB > k ⇒ TED > k` and the filter is sound.

use pgrx::prelude::*;

use crate::lb::sed::{bounded_sed, SEDIndex};
use crate::lb::ted::topdiff::ted_k;
use crate::types::UnifiedTreeIndex;

// ============================================================================
// Substrate → plain SEDIndex (String-labelled) construction
// ============================================================================

impl UnifiedTreeIndex {
    /// Build the plain, label-only `SEDIndex` (preorder + postorder label
    /// sequences) directly from the postorder substrate — no interning/dict.
    ///
    /// The postorder sequence is `labels` verbatim (the substrate stores nodes
    /// in postorder). The preorder sequence is recovered by reconstructing the
    /// topology from `{labels, sizes}` via the same sizes-stack trick used by
    /// `expand`, then a DFS from the root.
    ///
    /// The empty tree yields empty sequences; callers guard the empty case
    /// before Stage 1, so this is only defensive.
    fn sed_plain_build_sed_index(&self) -> SEDIndex {
        let n = self.tree_size;
        if n == 0 {
            return SEDIndex {
                preorder: Vec::new(),
                postorder: Vec::new(),
                tree_size: 0,
            };
        }

        // Reconstruct each node's children from postorder + subtree sizes. For
        // node i (postorder id), its `sizes[i] - 1` descendants are exactly the
        // stack entries whose sizes sum to that count (popped right→left).
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut stack: Vec<usize> = Vec::with_capacity(n);
        for i in 0..n {
            let sz = self.sizes[i] as usize;
            let mut remaining = sz - 1;
            let mut child_ids: Vec<usize> = Vec::new();
            while remaining > 0 {
                let c = *stack
                    .last()
                    .expect("stack underflow during topology reconstruction");
                let c_sz = self.sizes[c] as usize;
                stack.pop();
                child_ids.push(c);
                remaining -= c_sz;
            }
            // child_ids popped right→left; reverse for left→right order.
            child_ids.reverse();
            children[i] = child_ids;
            stack.push(i);
        }

        // Preorder DFS from the root (postorder id n-1): emit node, then children
        // left→right (push reversed so leftmost is popped first).
        let mut preorder: Vec<String> = Vec::with_capacity(n);
        let mut dfs_stack: Vec<usize> = vec![n - 1];
        while let Some(node) = dfs_stack.pop() {
            preorder.push(self.labels[node].clone());
            for &c in children[node].iter().rev() {
                dfs_stack.push(c);
            }
        }

        // Postorder is the substrate label order as-is.
        let postorder: Vec<String> = self.labels.clone();

        SEDIndex {
            preorder,
            postorder,
            tree_size: n,
        }
    }
}

// ============================================================================
// Pipeline: plain SED LB filter → exact bounded TopDiff
// ============================================================================

/// Two-stage tree-similarity pipeline over a single pre-indexed
/// `UnifiedTreeIndex` column, using a **plain** SED lower bound as the Stage-1
/// filter. Each argument's CBOR is deserialized once; Stage 1 builds the plain
/// label-only SED traversal indices directly from the substrate and runs the
/// bounded string edit distance. Only survivors reach Stage 2, which expands the
/// substrate into the TopDiff working form and runs exact bounded TopDiff.
///
/// Returns the TopDiff distance when the pair passes both stages (`<= k`),
/// otherwise `k + 1` (over-bound), composable with `<= k` filters in SQL.
/// Sound: plain SED on the label traversals is a true lower bound on TED, so
/// `LB > k ⇒ TED > k`.
#[pg_extern(immutable, parallel_safe, cost = 5000)]
fn sed_plain_topdiff_within(query: UnifiedTreeIndex, cand: UnifiedTreeIndex, k: i32) -> i32 {
    if k < 0 {
        return k + 1;
    }
    let k_usize = k as usize;
    // Stage 0 — size-diff gate (also covers empty/oversized-diff pairs).
    if query.tree_size.abs_diff(cand.tree_size) > k_usize {
        return k + 1;
    }
    // Empty-tree fast path: TED(∅, T) = |T| (all inserts/deletes). `bounded_sed`,
    // `expand` and `ted_k` are not defined for size-0 trees, so resolve here; the
    // size-diff gate above already returned k+1 when |T| exceeds k.
    if query.tree_size == 0 || cand.tree_size == 0 {
        return query.tree_size.max(cand.tree_size) as i32;
    }
    // Stage 1 — PLAIN bounded SED lower bound over label-only traversal
    // sequences, built directly from the substrate (no expand, no dict).
    //
    // Bound mapping: `bounded_sed(t1, t2, b)` returns the exact SED `d` when
    // `d <= b`, and a value `>= b + 1` (over-bound) when `d > b`. With budget
    // `b = k` the test `bounded_sed(..) > k` fires exactly when `d > k`, and
    // never when `d <= k` (which returns exactly `d <= k`). Since `d <= TED`,
    // firing proves `TED > k`, so filtering out is sound.
    let q_idx = query.sed_plain_build_sed_index();
    let c_idx = cand.sed_plain_build_sed_index();
    if bounded_sed(&q_idx, &c_idx, k_usize) > k_usize {
        return k + 1;
    }
    // Stage 2 — fresh shared label dictionary, expand into the TopDiff working
    // form (discard the SED-Struct halves), exact bounded TopDiff.
    let mut dict = rustc_hash::FxHashMap::default();
    let (_q_sed, q_td) = query.expand(&mut dict);
    let (_c_sed, c_td) = cand.expand(&mut dict);
    ted_k(&q_td, &c_td, k)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod sed_plain_unit_tests {
    use super::*;
    use crate::lb::sed::{bounded_sed, SEDIndex};
    use crate::parsing::parse_tree;
    use crate::types::{TreeArena, UnifiedTreeIndex};
    use std::ffi::CString;

    /// Same corpus as lib.rs `tests::TREES`: chains, bushy, deep; label overlap
    /// and size differences.
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
    fn empty() -> UnifiedTreeIndex {
        UnifiedTreeIndex {
            labels: vec![],
            sizes: vec![],
            tree_size: 0,
        }
    }

    /// Differential: the plain `SEDIndex` built from the substrate must produce
    /// the same traversal sequences AND the same `bounded_sed` results as the
    /// reference `SEDIndex::index_tree` over a parsed `TreeArena`.
    #[test]
    fn substrate_sed_index_matches_reference() {
        for &a in TREES {
            for &b in TREES {
                let my_q = uti(a).sed_plain_build_sed_index();
                let my_c = uti(b).sed_plain_build_sed_index();
                let ref_q = SEDIndex::index_tree(&ta(a));
                let ref_c = SEDIndex::index_tree(&ta(b));

                // Sequences identical (both are label-only, same traversal order).
                assert_eq!(my_q.preorder, ref_q.preorder, "preorder mismatch for {a}");
                assert_eq!(my_q.postorder, ref_q.postorder, "postorder mismatch for {a}");
                assert_eq!(my_q.tree_size, ref_q.tree_size, "tree_size mismatch for {a}");

                for &k in &[0usize, 1, 2, 3, 5, 10, 50] {
                    let mine = bounded_sed(&my_q, &my_c, k);
                    let reference = bounded_sed(&ref_q, &ref_c, k);
                    assert_eq!(
                        mine, reference,
                        "bounded_sed mismatch for ({a}, {b}, k={k}): {mine} != {reference}"
                    );
                }
            }
        }
    }

    /// Acceptance gate: the whole pipeline must equal exact APTED TED, capped at
    /// k+1. Validates Stage-1 soundness (never wrongly filters a within-k pair)
    /// and Stage-2 exactness together. `crate::tree_ed` is the APTED oracle.
    #[test]
    fn acceptance_gate_matches_exact_ted() {
        for &a in TREES {
            for &b in TREES {
                let exact = crate::tree_ed(ta(a), ta(b));
                for &k in &[0i32, 1, 2, 3, 7, 50] {
                    let expected = if exact <= k { exact } else { k + 1 };
                    let got = sed_plain_topdiff_within(uti(a), uti(b), k);
                    assert_eq!(
                        got, expected,
                        "sed_plain_topdiff_within({a}, {b}, {k}) = {got}, expected {expected} (exact TED {exact})"
                    );
                }
            }
        }
    }

    /// Stage-1 plain SED is a sound lower bound: computed with a large budget
    /// (nothing short-circuits), it never exceeds the exact TED.
    #[test]
    fn stage1_lb_is_sound() {
        for &a in TREES {
            for &b in TREES {
                let exact = crate::tree_ed(ta(a), ta(b));
                let q_idx = uti(a).sed_plain_build_sed_index();
                let c_idx = uti(b).sed_plain_build_sed_index();
                let lb = bounded_sed(&q_idx, &c_idx, 100_000) as i32;
                assert!(
                    lb <= exact,
                    "plain SED LB {lb} exceeds exact TED {exact} for ({a}, {b})"
                );
            }
        }
    }

    /// Edge cases: empty trees, negative k, identical trees at k=0.
    #[test]
    fn edge_cases() {
        // Two empty trees: TED 0.
        assert_eq!(sed_plain_topdiff_within(empty(), empty(), 3), 0);
        // Empty vs 3-node tree within budget: TED 3.
        assert_eq!(sed_plain_topdiff_within(empty(), uti("{a{b}{c}}"), 5), 3);
        // Empty vs 3-node tree, budget too small: k+1.
        assert_eq!(sed_plain_topdiff_within(empty(), uti("{a{b}{c}}"), 1), 2);
        // Empty on the candidate side too (symmetry).
        assert_eq!(sed_plain_topdiff_within(uti("{a{b}{c}}"), empty(), 5), 3);
        // Negative k is always over-bound (returns k+1).
        assert_eq!(sed_plain_topdiff_within(uti("{a}"), uti("{a}"), -1), 0);
        assert_eq!(sed_plain_topdiff_within(uti("{a{b}{c}}"), uti("{x{y}{z}}"), -3), -2);
        // Identical trees at k=0: TED 0.
        for &a in TREES {
            assert_eq!(
                sed_plain_topdiff_within(uti(a), uti(a), 0),
                0,
                "identical {a} at k=0 should be 0"
            );
        }
    }

    /// Off-by-one probes at the exact TED boundary. For a pair with exact TED
    /// `d >= 1`: `k = d` returns `d` (within budget) and `k = d - 1` returns
    /// `d` (over-bound `= (d-1)+1`). Exercises the Stage-1 bound mapping.
    #[test]
    fn off_by_one_probes() {
        for &a in TREES {
            for &b in TREES {
                let d = crate::tree_ed(ta(a), ta(b));
                if d < 1 {
                    continue;
                }
                assert_eq!(
                    sed_plain_topdiff_within(uti(a), uti(b), d),
                    d,
                    "k=d={d} should return d for ({a}, {b})"
                );
                assert_eq!(
                    sed_plain_topdiff_within(uti(a), uti(b), d - 1),
                    d,
                    "k=d-1={} should return d={d} (over-bound) for ({a}, {b})",
                    d - 1
                );
            }
        }
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    /// Round-trip through the SQL boundary: bracket-notation input function +
    /// CBOR arg serialization + the full plain-SED pipeline.
    #[pg_test]
    fn sed_plain_sql_round_trip() {
        let same = Spi::get_one::<i32>(
            "SELECT sed_plain_topdiff_within('{a{b}{c}}'::unifiedtreeindex, '{a{b}{c}}'::unifiedtreeindex, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(same, 0);
        let one = Spi::get_one::<i32>(
            "SELECT sed_plain_topdiff_within('{a{b}{c}}'::unifiedtreeindex, '{a{b}{x}}'::unifiedtreeindex, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(one, 1);
    }
}
