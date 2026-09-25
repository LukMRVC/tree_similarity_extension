//! Structural-filter LB → TopDiff verification pipeline.
//!
//! Sibling of `crate::sed_topdiff_within`: same two-stage contract, but Stage 1
//! is the STRUCTURAL FILTER lower bound (`crate::lb::structural_filter::ted`)
//! instead of the SED-Struct one. Stage 1 builds each input's `StructuralFilter`
//! straight from the `UnifiedTreeIndex` postorder substrate (label hashes +
//! subtree sizes), producing exactly what `LabelSetConverter::create` builds
//! from the parsed tree. Only survivors pay for `expand()` + exact bounded
//! TopDiff `ted_k`.

use pgrx::prelude::*;

use crate::lb::structural_filter::ted as structural_lb;
use crate::lb::ted::topdiff::ted_k;
use crate::types::tree_structural::{
    LabelSetElement, LabelSetElementBase, RegionNumType, StructHashMap, StructuralVec,
};
use crate::types::{StructuralFilter, Tree, UnifiedTreeIndex};

// ---------------------------------------------------------------------------
// Substrate helpers (inherent methods on UnifiedTreeIndex, `structural_`-prefixed
// to avoid colliding with the other pipeline agents' impl blocks in this crate).
// ---------------------------------------------------------------------------

impl UnifiedTreeIndex {
    /// Build the tree's `StructuralFilter` directly from the postorder
    /// substrate, equal to `StructuralFilter::from(tree)` on the parsed tree.
    ///
    /// For node `i` (postorder id `p = i + 1`, subtree size `s`, depth `d`) the
    /// regions are `[left, ancestors, right, descendants] =
    /// [p - s, d, n - (p + d), s - 1]`, the values `LabelSetConverter::
    /// create_record` computes during its traversal. Nodes are added to their
    /// label's set in postorder, as `create_record` does.
    fn structural_filter_from_substrate(&self) -> StructuralFilter {
        let n = self.tree_size;

        // Depth of every node. Node j's subtree spans postorder ids
        // [j + 1 - sizes[j], j], so scanning from the root (n - 1) downwards
        // with a stack of subtree start ids, the entries left after popping
        // those that do not contain i are exactly i's ancestors.
        let mut depth: Vec<RegionNumType> = vec![0; n];
        let mut ancestor_starts: Vec<usize> = Vec::new();
        for i in (0..n).rev() {
            while ancestor_starts.last().is_some_and(|&start| start > i) {
                ancestor_starts.pop();
            }
            depth[i] = ancestor_starts.len() as RegionNumType;
            ancestor_starts.push(i + 1 - self.sizes[i] as usize);
        }

        let tree_size = n as RegionNumType;
        let mut record_labels = StructHashMap::default();
        for i in 0..n {
            let label = self.labels[i];
            let postorder_id = i as RegionNumType + 1;
            let size = self.sizes[i];
            let d = depth[i];
            let node_struct_vec = StructuralVec {
                label_id: label,
                postorder_id,
                mapping_regions: [
                    postorder_id - size,
                    d,
                    tree_size - (postorder_id + d),
                    size - 1,
                ],
            };
            let se = record_labels.entry(label).or_insert_with(|| LabelSetElement {
                base: LabelSetElementBase {
                    id: label,
                    weight: 0,
                    ..LabelSetElementBase::default()
                },
                ..LabelSetElement::default()
            });
            se.base.weight += 1;
            se.struct_vec.push(node_struct_vec);
        }

        StructuralFilter(n, record_labels)
    }

    /// Stage-1 structural-filter lower bound on TED, computed purely from the
    /// substrate. Mirrors `crate::tree_lb_structural_filter(t1, t2, k)` exactly:
    /// the same size-diff gate, then `structural_lb(s1, s2, k)`. Sound:
    /// `structural_lb` is a true lower bound used at threshold `k`, so
    /// `LB > k ⇒ TED > k`.
    fn structural_stage1_lb(&self, cand: &UnifiedTreeIndex, k: i32) -> i32 {
        if self.tree_size.abs_diff(cand.tree_size) as i32 > k {
            return k + 1;
        }
        let s1 = self.structural_filter_from_substrate();
        let s2 = cand.structural_filter_from_substrate();
        structural_lb(&s1, &s2, k)
    }
}

/// Combined Structural-filter LB → TopDiff verification pipeline over a single
/// pre-indexed `UnifiedTreeIndex` column. Stage 1 builds each argument's
/// structural filter from its postorder substrate and runs the cheap structural
/// lower bound; only survivors are `expand`ed into the TopDiff working form and
/// verified with exact bounded TopDiff.
///
/// Returns the TopDiff distance when the pair passes both stages (`<= k`),
/// otherwise `k + 1` (over-bound), composable with `<= k` filters in SQL.
/// Sound: the structural filter is a true lower bound on TED, so `LB > k ⇒
/// TED > k`.
#[pg_extern(immutable, parallel_safe, cost = 5000)]
fn structural_topdiff_within(query: Tree, cand: Tree, k: i32) -> i32 {
    structural_within(&query.to_unified(), &cand.to_unified(), k)
}

pub fn structural_within(query: &UnifiedTreeIndex, cand: &UnifiedTreeIndex, k: i32) -> i32 {
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
    // Stage 1 — structural-filter lower bound, straight from the substrate (no
    // expand yet). Filtered out — without ever building the TopDiff form — if the
    // LB proves TED > k.
    if query.structural_stage1_lb(&cand, k) > k {
        return k + 1;
    }
    // Stage 2 — survivors only: exact bounded TopDiff (already returns k+1 on
    // over-bound). The SED halves of `expand` are discarded.
    let (_q_sed, q_td) = query.expand();
    let (_c_sed, c_td) = cand.expand();
    ted_k(&q_td, &c_td, k)
}

// ===========================================================================
// Unit tests — run WITHOUT Postgres. `#[pg_extern]` fns are plain Rust fns, so
// they're callable directly here.
// ===========================================================================
#[cfg(test)]
mod structural_unit_tests {
    use crate::parsing::parse_tree;
    use crate::types::{StructuralFilter, TreeArena, UnifiedTreeIndex};
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
    fn tr(s: &str) -> crate::types::Tree {
        crate::types::Tree::parse(s).unwrap()
    }

    /// Construction differential: the filter built from the substrate must equal
    /// the one `LabelSetConverter` builds from the parsed tree — same label sets,
    /// weights, postorder ids and region vectors, in the same order.
    #[test]
    fn substrate_filter_matches_label_set_converter() {
        let repeated_labels = [
            "{a{b}{a{b}{c}{a}}{b}}",
            "{a{c}{b{a{a}{b}{c}}}}",
            "{a{a{a{a}}}}",
            "{a{a}{a}{a{a}{a}}}",
        ];
        for &s in TREES.iter().chain(repeated_labels.iter()) {
            let mine = uti(s).structural_filter_from_substrate();
            let reference = StructuralFilter::from(ta(s));
            assert_eq!(mine, reference, "structural filter mismatch for {s}");
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
                    let reference = crate::tree_lb_structural_filter(tr(a), tr(b), k);
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
                let exact = crate::tree_ed(tr(a), tr(b));
                for &k in &[0i32, 1, 2, 3, 7, 50] {
                    let expected = if exact <= k { exact } else { k + 1 };
                    let got = super::structural_within(&uti(a), &uti(b), k);
                    assert_eq!(
                        got, expected,
                        "structural_within({a}, {b}, {k}) = {got}, expected {expected} (exact TED {exact})"
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
                let exact = crate::tree_ed(tr(a), tr(b));
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
        assert_eq!(super::structural_within(&empty(), &empty(), 3), 0);
        // Empty vs 3-node tree within budget: TED 3.
        assert_eq!(
            super::structural_within(&empty(), &uti("{a{b}{c}}"), 5),
            3
        );
        // Empty vs 3-node tree, budget too small: k+1.
        assert_eq!(
            super::structural_within(&empty(), &uti("{a{b}{c}}"), 1),
            2
        );
        // Negative k is always over-bound (returns k+1).
        assert_eq!(super::structural_within(&uti("{a}"), &uti("{a}"), -1), 0);
        // Identical trees at k=0: exact 0.
        assert_eq!(
            super::structural_within(&uti("{a{b}{c}}"), &uti("{a{b}{c}}"), 0),
            0
        );

        // Off-by-one probes: pick a pair with exact TED d >= 1. At k=d the exact
        // value comes through; at k=d-1 it is capped to k+1 == d (not d-1, not
        // d+1) — an over/under-bound in the boundary would show up here.
        let a = "{a{b}{c}}";
        let b = "{x{y}{z}}";
        let d = crate::tree_ed(tr(a), tr(b));
        assert!(d >= 1, "expected a non-trivial TED for the probe pair");
        assert_eq!(super::structural_within(&uti(a), &uti(b), d), d);
        assert_eq!(
            super::structural_within(&uti(a), &uti(b), d - 1),
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
            "SELECT structural_topdiff_within('{a{b}{c}}'::tree, '{a{b}{c}}'::tree, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(same, 0);
        let one = Spi::get_one::<i32>(
            "SELECT structural_topdiff_within('{a{b}{c}}'::tree, '{a{b}{x}}'::tree, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(one, 1);
    }
}
