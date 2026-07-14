//! Structural-filter LB → TopDiff verification pipeline.
//!
//! Sibling of `crate::sed_topdiff_within`: same two-stage contract, but Stage 1
//! is the STRUCTURAL FILTER lower bound (`crate::lb::structural_filter::ted`)
//! instead of the SED-Struct one. Stage 1 is computed straight from the
//! `UnifiedTreeIndex` postorder substrate by reconstructing a `TreeArena` for
//! each input (topology recovered from the {postorder id, subtree size} pair via
//! the sizes-stack trick) and feeding both through ONE `StructuralSetConverter`
//! (shared label universe), exactly as `crate::tree_lb_structural_filter` does.
//! Only survivors pay for `expand()` + exact bounded TopDiff `ted_k`.

use pgrx::prelude::*;

use crate::lb::structural_filter::ted as structural_lb;
use crate::lb::ted::topdiff::ted_k;
use crate::types::tree_internals::id::NodeId;
use crate::types::{StructuralSetConverter, TreeArena, UnifiedTreeIndex};

// ---------------------------------------------------------------------------
// Substrate helpers (inherent methods on UnifiedTreeIndex, `structural_`-prefixed
// to avoid colliding with the other pipeline agents' impl blocks in this crate).
// ---------------------------------------------------------------------------

impl UnifiedTreeIndex {
    /// Reconstruct a `TreeArena` from the postorder `{labels, sizes}` substrate.
    ///
    /// Postorder ids together with subtree sizes uniquely determine the topology:
    /// scanning postorder with a stack, node `i`'s direct children are the stack
    /// entries whose sizes sum to `sizes[i] - 1` (popped right→left, so reversed
    /// to left→right). The root (postorder `n-1`) is created FIRST so it lands at
    /// arena index 0 — `StructuralSetConverter` takes `tree.iter().next()` as the
    /// root — and children are appended left→right so postorder numbering matches
    /// the originally parsed tree. Empty substrate yields an empty arena.
    fn structural_rebuild_arena(&self) -> TreeArena {
        let n = self.tree_size;
        let mut arena = TreeArena::with_capacity(n);
        if n == 0 {
            return arena;
        }

        // children[i] = left→right postorder ids of node i's direct children.
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
            child_ids.reverse(); // right→left pop order → left→right children
            children[i] = child_ids;
            stack.push(i);
        }

        // Preorder build so every parent exists (and the root is created first,
        // landing at index 0) before its children are attached.
        let root_post = n - 1;
        let mut post_to_nid: Vec<Option<NodeId>> = vec![None; n];
        let root_nid = arena.new_node(self.labels[root_post].clone());
        post_to_nid[root_post] = Some(root_nid);

        let mut dfs: Vec<usize> = vec![root_post];
        while let Some(p) = dfs.pop() {
            let p_nid = post_to_nid[p].expect("parent created before its children");
            for &c in &children[p] {
                let c_nid = arena.new_node(self.labels[c].clone());
                post_to_nid[c] = Some(c_nid);
                p_nid.append(c_nid, &mut arena);
            }
            for &c in children[p].iter().rev() {
                dfs.push(c);
            }
        }

        arena
    }

    /// Stage-1 structural-filter lower bound on TED, computed purely from the
    /// substrate. Mirrors `crate::tree_lb_structural_filter(t1, t2, k)` exactly:
    /// the same size-diff gate, ONE shared `StructuralSetConverter` for both
    /// trees, and `structural_lb(s1, s2, k)`. Sound: `structural_lb` is a true
    /// lower bound used at threshold `k`, so `LB > k ⇒ TED > k`.
    ///
    /// Both trees MUST be non-empty — `StructuralSetConverter::create` panics on
    /// an empty tree; the pipeline's stage-0 gates guarantee this before the call.
    fn structural_stage1_lb(&self, cand: &UnifiedTreeIndex, k: i32) -> i32 {
        if self.tree_size.abs_diff(cand.tree_size) as i32 > k {
            return k + 1;
        }
        let t1 = self.structural_rebuild_arena();
        let t2 = cand.structural_rebuild_arena();
        let mut lsc = StructuralSetConverter::default();
        let tuples = lsc.create(&[t1, t2]);
        match &tuples[..2] {
            [s1, s2] => structural_lb(s1, s2, k),
            _ => panic!("Trees failed to convert!"),
        }
    }
}

/// Combined Structural-filter LB → TopDiff verification pipeline over a single
/// pre-indexed `UnifiedTreeIndex` column. Stage 1 reconstructs a `TreeArena` for
/// each argument from its postorder substrate and runs the cheap structural
/// lower bound; only survivors are `expand`ed into the TopDiff working form and
/// verified with exact bounded TopDiff.
///
/// Returns the TopDiff distance when the pair passes both stages (`<= k`),
/// otherwise `k + 1` (over-bound), composable with `<= k` filters in SQL.
/// Sound: the structural filter is a true lower bound on TED, so `LB > k ⇒
/// TED > k`.
#[pg_extern(immutable, parallel_safe, cost = 5000)]
fn structural_topdiff_within(query: UnifiedTreeIndex, cand: UnifiedTreeIndex, k: i32) -> i32 {
    if k < 0 {
        return k + 1;
    }
    let k_usize = k as usize;
    // Stage 0 — size-diff gate (also covers empty/oversized-diff pairs).
    if query.tree_size.abs_diff(cand.tree_size) > k_usize {
        return k + 1;
    }
    // Empty-tree fast path: TED(∅, T) = |T| (all inserts/deletes). The structural
    // converter and `ted_k` are not defined for size-0 trees, so resolve here;
    // the size-diff gate above already returned k+1 when |T| exceeds k.
    if query.tree_size == 0 || cand.tree_size == 0 {
        return query.tree_size.max(cand.tree_size) as i32;
    }
    // Stage 1 — structural-filter lower bound, straight from the substrate (no
    // expand yet). Filtered out — without ever building the TopDiff form — if the
    // LB proves TED > k.
    if query.structural_stage1_lb(&cand, k) > k {
        return k + 1;
    }
    // Stage 2 — survivors only: one shared label dictionary across both trees so
    // label ids agree, then exact bounded TopDiff (already returns k+1 on
    // over-bound). The SED halves of `expand` are discarded.
    let mut dict = rustc_hash::FxHashMap::default();
    let (_q_sed, q_td) = query.expand(&mut dict);
    let (_c_sed, c_td) = cand.expand(&mut dict);
    ted_k(&q_td, &c_td, k)
}

// ===========================================================================
// Unit tests — run WITHOUT Postgres. `#[pg_extern]` fns are plain Rust fns, so
// they're callable directly here.
// ===========================================================================
#[cfg(test)]
mod structural_unit_tests {
    use crate::parsing::parse_tree;
    use crate::types::{TreeArena, UnifiedTreeIndex};
    use std::ffi::CString;

    /// Same corpus as the lib.rs acceptance tests: varied shapes (chains, bushy,
    /// deep), label overlap, and size differences.
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

    /// Reconstruction differential: rebuilding a `TreeArena` from the substrate
    /// and re-deriving a `UnifiedTreeIndex` from it reproduces the exact same
    /// {labels, sizes, tree_size}. Proves the topology recovery is faithful.
    #[test]
    fn reconstruction_roundtrip() {
        for &s in TREES {
            let u = uti(s);
            let rebuilt = u.structural_rebuild_arena();
            let u2 = UnifiedTreeIndex::from(rebuilt);
            assert_eq!(u.labels, u2.labels, "labels mismatch for {s}");
            assert_eq!(u.sizes, u2.sizes, "sizes mismatch for {s}");
            assert_eq!(u.tree_size, u2.tree_size, "tree_size mismatch for {s}");
        }
    }

    /// LB differential: the substrate-based Stage-1 LB must equal the reference
    /// `tree_lb_structural_filter` on directly-parsed trees, for every pair and k.
    /// An off-by-one in the bound/return mapping breaks this.
    #[test]
    fn lb_matches_reference_structural_filter() {
        for &a in TREES {
            for &b in TREES {
                for &k in &[0i32, 1, 2, 3, 7, 50] {
                    let mine = uti(a).structural_stage1_lb(&uti(b), k);
                    let reference = crate::tree_lb_structural_filter(ta(a), ta(b), k);
                    assert_eq!(
                        mine, reference,
                        "stage-1 LB mismatch ({a}, {b}, k={k}): {mine} != {reference}"
                    );
                }
            }
        }
    }

    /// Acceptance gate: the whole pipeline must equal exact APTED TED, capped at
    /// k+1. Validates Stage-1 soundness (never wrongly filters a within-k pair)
    /// AND Stage-2 exactness together. `tree_ed` is the independent APTED oracle.
    #[test]
    fn within_matches_exact_ted() {
        for &a in TREES {
            for &b in TREES {
                let exact = crate::tree_ed(ta(a), ta(b));
                for &k in &[0i32, 1, 2, 3, 7, 50] {
                    let expected = if exact <= k { exact } else { k + 1 };
                    let got = super::structural_topdiff_within(uti(a), uti(b), k);
                    assert_eq!(
                        got, expected,
                        "structural_topdiff_within({a}, {b}, {k}) = {got}, expected {expected} (exact TED {exact})"
                    );
                }
            }
        }
    }

    /// Stage-1 soundness: with a large bound (nothing short-circuits) the
    /// structural LB never exceeds the exact TED. If it did, Stage 1 could wrongly
    /// filter a true match.
    #[test]
    fn stage1_lb_is_sound() {
        for &a in TREES {
            for &b in TREES {
                let exact = crate::tree_ed(ta(a), ta(b));
                let lb = uti(a).structural_stage1_lb(&uti(b), 1000);
                assert!(
                    lb <= exact,
                    "structural LB {lb} exceeds exact TED {exact} for ({a}, {b})"
                );
            }
        }
    }

    /// Edge cases and off-by-one probes around the budget boundary.
    #[test]
    fn edge_cases() {
        let empty = || UnifiedTreeIndex {
            labels: vec![],
            sizes: vec![],
            tree_size: 0,
        };
        // Two empty trees: TED 0.
        assert_eq!(super::structural_topdiff_within(empty(), empty(), 3), 0);
        // Empty vs 3-node tree within budget: TED 3.
        assert_eq!(
            super::structural_topdiff_within(empty(), uti("{a{b}{c}}"), 5),
            3
        );
        // Empty vs 3-node tree, budget too small: k+1.
        assert_eq!(
            super::structural_topdiff_within(empty(), uti("{a{b}{c}}"), 1),
            2
        );
        // Negative k is always over-bound (returns k+1).
        assert_eq!(super::structural_topdiff_within(uti("{a}"), uti("{a}"), -1), 0);
        // Identical trees at k=0: exact 0.
        assert_eq!(
            super::structural_topdiff_within(uti("{a{b}{c}}"), uti("{a{b}{c}}"), 0),
            0
        );

        // Off-by-one probes: pick a pair with exact TED d >= 1. At k=d the exact
        // value comes through; at k=d-1 it is capped to k+1 == d (not d-1, not
        // d+1) — an over/under-bound in the boundary would show up here.
        let a = "{a{b}{c}}";
        let b = "{x{y}{z}}";
        let d = crate::tree_ed(ta(a), ta(b));
        assert!(d >= 1, "expected a non-trivial TED for the probe pair");
        assert_eq!(super::structural_topdiff_within(uti(a), uti(b), d), d);
        assert_eq!(
            super::structural_topdiff_within(uti(a), uti(b), d - 1),
            d,
            "k = d-1 must over-bound to k+1 = d"
        );
    }
}

// ===========================================================================
// Postgres SPI round-trip — one #[pg_test]. Module name is globally unique.
// ===========================================================================
#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    /// Round-trip through the SQL boundary: bracket-notation input function +
    /// CBOR arg serialization + the full pipeline. Function name is globally
    /// unique because `#[pg_test]` derives an FFI wrapper symbol from it.
    #[pg_test]
    fn structural_sql_round_trip() {
        let same = Spi::get_one::<i32>(
            "SELECT structural_topdiff_within('{a{b}{c}}'::unifiedtreeindex, '{a{b}{c}}'::unifiedtreeindex, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(same, 0);
        let one = Spi::get_one::<i32>(
            "SELECT structural_topdiff_within('{a{b}{c}}'::unifiedtreeindex, '{a{b}{x}}'::unifiedtreeindex, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(one, 1);
    }
}
