//! Binary-branch LB filter → TopDiff verification pipeline.
//!
//! Stage 1 is a faithful port of the binary-branch lower bound from
//! `ted-search/lib/ted-lb-bib` (`BinaryBranchAlgorithm`, implementing the
//! `LowerBoundMethod` trait in `ted-search/lib/ted-base`). The algorithm is the
//! classic binary-branch tree-edit-distance lower bound of Yang, Kalnis & Tung,
//! "Similarity Evaluation on Tree-structured Data" (SIGMOD 2005): each node maps
//! to a binary-branch triple `(label, first-child-label, next-sibling-label)` in
//! the left-child/right-sibling binarization, and the L1 distance of the two
//! branch-count vectors is `<= 5 * TED` (DIVISOR = 5), hence `TED >= dist/5`.
//!
//! Adaptations for this extension:
//!   * `ted-lb-bib` walks an `indextree::Arena<LabelId>` (`ParsedTree`, labels
//!     pre-interned to `i32`). Here we derive the same binary branches straight
//!     from the `UnifiedTreeIndex` postorder substrate (label hashes + subtree
//!     sizes): the topology (children lists) is reconstructed with the same
//!     stack scan `expand` uses, and the stored label hashes play the role of
//!     `LabelId`.
//!   * The source interns every branch triple to an `i32` id in a table shared
//!     by the two trees. Label hashes are global ids, so a triple of hashes is
//!     itself a global key: branch vectors are keyed by the triple directly and
//!     each tree is preprocessed on its own, with no shared state.
//!   * The source's `itertools` usage (`collect_vec`) is replaced with plain std
//!     iterators (this crate does not depend on `itertools`).

use pgrx::prelude::*;
use rustc_hash::FxHashMap;

use crate::lb::ted::topdiff::ted_k;
use crate::parsing::LabelHash;
use crate::types::{Tree, UnifiedTreeIndex};

// ============================================================================
// Ported binary-branch converter + lower bound (from ted-lb-bib)
// ============================================================================

/// Multiset of binary branches: branch triple -> occurrence count.
/// (`ted-lb-bib::BinaryBranchVector`, keyed by the triple instead of an id.)
type BinaryBranchVector = FxHashMap<BbTuple, i32>;

/// A preprocessed tree: node count + its binary-branch vector.
/// (`ted-lb-bib::BinaryBranchTree`.)
struct BinaryBranchTree {
    size: usize,
    branch_vector: BinaryBranchVector,
}

/// Binary-branch triple `(root label, left-child label, next-sibling label)`,
/// each a label hash. (`ted-lb-bib::BBTuple`.)
type BbTuple = (LabelHash, Option<LabelHash>, Option<LabelHash>);

/// Divisor from the binary-branch theorem: `L1(branch vectors) <= DIVISOR * TED`.
/// (`ted-base::LowerBoundMethod::DIVISOR` = 5 for `BinaryBranchAlgorithm`.)
const BIB_DIVISOR: usize = 5;

/// Port of `BinaryBranchAlgorithm::preprocess` + `create_vector` for a single
/// tree, computed from the `UnifiedTreeIndex` substrate instead of walking an
/// `indextree::Arena`.
///
/// For each node `i` (postorder), the binary branch is
/// `(label(i), label(first child of i), label(next sibling of i))` — matching
/// the source, where `create_vector` passes each child its immediate right
/// sibling's label and reads its own first child's label.
fn bib_preprocess(uti: &UnifiedTreeIndex) -> BinaryBranchTree {
    let n = uti.tree_size;
    let mut branch_vector: BinaryBranchVector = FxHashMap::default();
    if n == 0 {
        return BinaryBranchTree { size: 0, branch_vector };
    }
    for tuple in uti.bib_branch_tuples() {
        *branch_vector.entry(tuple).or_insert(0) += 1;
    }
    BinaryBranchTree { size: n, branch_vector }
}

/// Port of `BinaryBranchAlgorithm::lower_bound`. Returns the binary-branch
/// distance `L1(γ) = (|d| + |q|) - 2 * |γ(q) ∩ γ(d)|` (a multiset symmetric
/// difference), OR the sentinel `threshold * DIVISOR + 1` when the node counts
/// differ by more than `threshold` (a node-count-difference LB on TED).
///
/// Consumer contract (from `ted-search`): a candidate is WITHIN the threshold iff
/// `lower_bound <= threshold * DIVISOR`; the sentinel is the least value that
/// fails that test. Equivalently `TED >= ⌈dist / DIVISOR⌉`, so filtering iff
/// `dist > DIVISOR * threshold` is sound.
fn bib_lower_bound(query: &BinaryBranchTree, data: &BinaryBranchTree, threshold: usize) -> usize {
    let (t1s, t2s) = (data.size, query.size);
    if t1s.abs_diff(t2s) > threshold {
        return threshold * BIB_DIVISOR + 1;
    }
    let mut intersection_size = 0usize;
    for (label, postings) in data.branch_vector.iter() {
        let Some(q_postings) = query.branch_vector.get(label) else {
            continue;
        };
        intersection_size += (*q_postings).min(*postings) as usize;
    }
    (t1s + t2s) - (2 * intersection_size)
}

impl UnifiedTreeIndex {
    /// The binary-branch triple of every node, in postorder.
    fn bib_branch_tuples(&self) -> Vec<BbTuple> {
        let n = self.tree_size;
        let children = self.bib_children();

        // right_sib[v] = the node immediately to v's right under the same parent.
        let mut right_sib: Vec<Option<usize>> = vec![None; n];
        for kids in &children {
            for w in kids.windows(2) {
                right_sib[w[0]] = Some(w[1]);
            }
        }

        (0..n)
            .map(|i| {
                let left = children[i].first().map(|&c| self.labels[c]);
                let right = right_sib[i].map(|c| self.labels[c]);
                (self.labels[i], left, right)
            })
            .collect()
    }

    /// Reconstruct the left→right children list of every node from the postorder
    /// `{labels, sizes}` substrate, using the same stack scan as `expand`: node
    /// `i`'s direct children are the stack entries whose subtree sizes sum to
    /// `sizes[i] - 1` (popped right→left, then reversed to left→right).
    fn bib_children(&self) -> Vec<Vec<usize>> {
        let n = self.tree_size;
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
            child_ids.reverse();
            children[i] = child_ids;
            stack.push(i);
        }
        children
    }
}

// ============================================================================
// Pipeline
// ============================================================================

/// Combined binary-branch LB filter → TopDiff verification pipeline over a single
/// pre-indexed `UnifiedTreeIndex` column. Each argument's CBOR is deserialized
/// once; Stage 1 derives the binary-branch vectors straight from the postorder
/// substrate and Stage 2 — only on survivors — expands into the TopDiff form and
/// runs exact bounded TopDiff.
///
/// Stage 1 is the binary-branch lower bound (Yang, Kalnis & Tung, SIGMOD 2005;
/// ported from `ted-search`'s `ted-lb-bib::BinaryBranchAlgorithm`). Each node
/// becomes a `(label, first-child-label, next-sibling-label)` triple in the
/// left-child/right-sibling binarization, and the branch-vector L1 distance
/// `dist = |q| + |c| - 2·|γ(q) ∩ γ(c)|` satisfies `dist <= 5·TED` (DIVISOR = 5),
/// so `TED >= ⌈dist/5⌉`. The pair is filtered (TED > k proven, `expand` skipped)
/// iff `dist > 5·k`.
///
/// Returns the exact TopDiff distance when the pair passes both stages (`<= k`),
/// otherwise `k + 1` (over-bound), composable with `<= k` filters in SQL.
/// Sound: the binary-branch distance is a true lower bound on TED, so
/// `dist > 5·k ⇒ TED > k`.
#[pg_extern(immutable, parallel_safe, cost = 5000)]
fn binary_branch_topdiff_within(query: Tree, cand: Tree, k: i32) -> i32 {
    binary_branch_within(&query.to_unified(), &cand.to_unified(), k)
}

pub fn binary_branch_within(query: &UnifiedTreeIndex, cand: &UnifiedTreeIndex, k: i32) -> i32 {
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
    // Stage 1 — binary-branch lower bound. Filtered iff dist > 5·k.
    let q_bb = bib_preprocess(&query);
    let c_bb = bib_preprocess(&cand);
    if bib_lower_bound(&q_bb, &c_bb, k_usize) > BIB_DIVISOR * k_usize {
        return k + 1;
    }
    // Stage 2 — exact bounded TopDiff. Discard the SED-struct halves.
    let (_q_sed, q_td) = query.expand();
    let (_c_sed, c_td) = cand.expand();
    ted_k(&q_td, &c_td, k)
}

// ============================================================================
// Tests
// ============================================================================

/// Test-only cross-check: the same distance computed as an explicit L1 sum over
/// the two branch vectors (port of `tree-statistics`'s `ted_l1`). Must agree with
/// `bib_lower_bound` (both are the multiset symmetric difference).
#[cfg(test)]
fn bib_lower_bound_l1(query: &BinaryBranchTree, data: &BinaryBranchTree, threshold: usize) -> usize {
    let (t1s, t2s) = (data.size, query.size);
    if t1s.abs_diff(t2s) > threshold {
        return threshold + 1;
    }
    let mut dist = 0i32;
    for (label, postings) in data.branch_vector.iter() {
        let q = query.branch_vector.get(label).copied().unwrap_or(0);
        dist += (postings - q).abs();
    }
    for (label, postings) in query.branch_vector.iter() {
        if !data.branch_vector.contains_key(label) {
            dist += *postings;
        }
    }
    dist as usize
}

#[cfg(test)]
mod binary_branch_unit_tests {
    use super::*;
    use crate::parsing::{label_hash, parse_tree};
    use crate::types::TreeArena;
    use std::ffi::CString;

    /// Same corpus as lib.rs's `tests::TREES`.
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
    fn empty() -> UnifiedTreeIndex {
        UnifiedTreeIndex { labels: vec![], sizes: vec![], tree_size: 0 }
    }
    fn h(label: &str) -> LabelHash {
        label_hash(label.as_bytes())
    }

    /// `⌈dist / DIVISOR⌉` — the actual integer lower bound on TED that the
    /// binary-branch distance implies.
    fn lb_from_dist(dist: usize) -> usize {
        (dist + BIB_DIVISOR - 1) / BIB_DIVISOR
    }

    // -----------------------------------------------------------------------
    // Port fidelity: hand-computed binarization triples.
    // -----------------------------------------------------------------------

    /// Singleton `{a}`: root has no child and no sibling → `(a, None, None)`.
    #[test]
    fn triples_singleton() {
        let tuples = uti("{a}").bib_branch_tuples();
        assert_eq!(tuples, vec![(h("a"), None, None)]);
    }

    /// `{a{b}{c}}` (postorder b, c, a):
    ///   b — leaf, next sibling c   → (b, None, Some c)
    ///   c — leaf, last child       → (c, None, None)
    ///   a — first child b, root    → (a, Some b, None)
    #[test]
    fn triples_bushy() {
        let tuples = uti("{a{b}{c}}").bib_branch_tuples();
        assert_eq!(
            tuples,
            vec![
                (h("b"), None, Some(h("c"))),
                (h("c"), None, None),
                (h("a"), Some(h("b")), None),
            ]
        );
    }

    /// `{a{b{c}}}` chain (postorder c, b, a):
    ///   c — leaf, only child       → (c, None, None)
    ///   b — first child c, only    → (b, Some c, None)
    ///   a — first child b, root    → (a, Some b, None)
    #[test]
    fn triples_chain() {
        let tuples = uti("{a{b{c}}}").bib_branch_tuples();
        assert_eq!(
            tuples,
            vec![
                (h("c"), None, None),
                (h("b"), Some(h("c")), None),
                (h("a"), Some(h("b")), None),
            ]
        );
    }

    /// Mirror of the source repo's `test_binary_branch_converter`: two trees, the
    /// binary-branch distance (== its explicit L1 form) is `<= 15`.
    #[test]
    fn source_example_distance() {
        let a = uti("{a{b{c}{d}}{b{c}{d}}{e}}");
        let b = uti("{a{b{c}{d}{b{e}}}{c}{d}{e}}");
        let a_bb = bib_preprocess(&a);
        let b_bb = bib_preprocess(&b);

        let sym = bib_lower_bound(&a_bb, &b_bb, usize::MAX);
        let l1 = bib_lower_bound_l1(&a_bb, &b_bb, usize::MAX);
        assert_eq!(sym, l1, "symmetric-diff and L1 forms must agree");
        assert!(sym <= 15, "distance {sym} should be <= 15");
    }

    /// Branch-vector counts sum to the node count (each node contributes exactly
    /// one triple), and the distance of a tree with itself is 0.
    #[test]
    fn self_distance_zero_and_vector_totals() {
        for &s in TREES {
            let t = uti(s);
            let bb = bib_preprocess(&t);
            let total: i32 = bb.branch_vector.values().sum();
            assert_eq!(total as usize, t.tree_size, "vector total != node count for {s}");
            // Self-comparison: identical vectors → distance 0.
            let x = bib_preprocess(&uti(s));
            let y = bib_preprocess(&uti(s));
            assert_eq!(bib_lower_bound(&x, &y, usize::MAX), 0, "self distance != 0 for {s}");
        }
    }

    // -----------------------------------------------------------------------
    // LB soundness: ⌈dist/5⌉ <= exact TED for every corpus pair.
    // -----------------------------------------------------------------------

    #[test]
    fn lb_is_sound() {
        for &a in TREES {
            for &b in TREES {
                let exact = crate::tree_ed(tr(a), tr(b)) as usize;
                let a_bb = bib_preprocess(&uti(a));
                let b_bb = bib_preprocess(&uti(b));
                // Huge threshold so the sentinel never fires: raw distance.
                let dist = bib_lower_bound(&a_bb, &b_bb, usize::MAX);
                let lb = lb_from_dist(dist);
                assert!(
                    lb <= exact,
                    "binary-branch LB {lb} (dist {dist}) exceeds exact TED {exact} for ({a}, {b})"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Acceptance gate: whole pipeline == exact APTED TED, capped at k+1.
    // -----------------------------------------------------------------------

    #[test]
    fn within_matches_exact_ted() {
        for &a in TREES {
            for &b in TREES {
                let exact = crate::tree_ed(tr(a), tr(b));
                for &k in &[0i32, 1, 2, 3, 7, 50] {
                    let expected = if exact <= k { exact } else { k + 1 };
                    let got = binary_branch_within(&uti(a), &uti(b), k);
                    assert_eq!(
                        got, expected,
                        "binary_branch_within({a}, {b}, {k}) = {got}, expected {expected} (exact TED {exact})"
                    );
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Edge cases + off-by-one probes.
    // -----------------------------------------------------------------------

    #[test]
    fn edge_cases() {
        // Two empty trees: TED 0.
        assert_eq!(binary_branch_within(&empty(), &empty(), 3), 0);
        // Empty vs 3-node tree within budget: TED 3.
        assert_eq!(binary_branch_within(&empty(), &uti("{a{b}{c}}"), 5), 3);
        // Empty vs 3-node tree, budget too small: k+1.
        assert_eq!(binary_branch_within(&empty(), &uti("{a{b}{c}}"), 1), 2);
        // Negative k is always over-bound (returns k+1).
        assert_eq!(binary_branch_within(&uti("{a}"), &uti("{a}"), -1), 0);
        // Identical trees at k=0: exact 0.
        assert_eq!(binary_branch_within(&uti("{a{b}{c}}"), &uti("{a{b}{c}}"), 0), 0);
    }

    /// For a pair with exact TED d >= 1: k=d returns d, k=d-1 returns d (=k+1).
    #[test]
    fn off_by_one_probes() {
        for (a, b) in [
            ("{a{b}{c}}", "{a{b}{x}}"), // one relabel: d = 1
            ("{a}", "{a{b}{c}}"),       // two inserts:  d = 2
            ("{a{b}{c}}", "{x{y}{z}}"), // three relabels: d = 3
        ] {
            let d = crate::tree_ed(tr(a), tr(b));
            assert!(d >= 1, "expected TED >= 1 for ({a}, {b}), got {d}");
            assert_eq!(
                binary_branch_within(&uti(a), &uti(b), d),
                d,
                "k=d exact expected for ({a}, {b}), d={d}"
            );
            assert_eq!(
                binary_branch_within(&uti(a), &uti(b), d - 1),
                d, // == (d-1) + 1 == k+1
                "k=d-1 over-bound expected for ({a}, {b}), d={d}"
            );
        }
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    /// Round-trip through the SQL boundary: bracket-notation input function +
    /// CBOR arg serialization + the full pipeline. Unique fn name to avoid the
    /// global `pg_finfo_*_wrapper` symbol colliding with other pg_tests.
    #[pg_test]
    fn binary_branch_sql_round_trip() {
        let same = Spi::get_one::<i32>(
            "SELECT binary_branch_topdiff_within('{a{b}{c}}'::tree, '{a{b}{c}}'::tree, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(same, 0);
        let one = Spi::get_one::<i32>(
            "SELECT binary_branch_topdiff_within('{a{b}{c}}'::tree, '{a{b}{x}}'::tree, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(one, 1);
    }
}
