//! Label-intersection LB filter → TopDiff verification pipeline.
//!
//! Two-stage similarity pipeline over a single pre-indexed `UnifiedTreeIndex`
//! column, mirroring `sed_topdiff_within` (lib.rs) but swapping Stage 1 for the
//! *label-intersection* lower bound. Stage 1 is computed directly from the
//! postorder `labels` multiset the substrate already carries — no `TreeArena`
//! reconstruction and, crucially, no `expand()` call unless the pair survives.

use pgrx::prelude::*;

use crate::lb::ted::topdiff::ted_k;
use crate::types::UnifiedTreeIndex;

// ---------------------------------------------------------------------------
// Stage 1 — multiset label-intersection lower bound, straight off the substrate.
// ---------------------------------------------------------------------------

impl UnifiedTreeIndex {
    /// Multiset label-intersection lower bound on TED, computed directly from the
    /// two postorder `labels` multisets: `max(|q|, |c|) - |labels(q) ∩ labels(c)|`
    /// where the intersection is the MULTISET intersection
    /// `Σ_label min(count_q, count_c)` — exactly the statistic `InvertedTree`
    /// holds and `inverted_lblint` computes.
    ///
    /// This is a true lower bound on the tree edit distance (a matched-label node
    /// is the only kind that can survive without an insert/delete/rename), so
    /// `LB > k ⇒ TED > k`, making it a sound Stage-1 filter.
    ///
    /// Bounded/short-circuiting: returns the exact LB when it is `<= k`, otherwise
    /// `k + 1` (the over-bound signal, matching `bounded_label_intersection_distance`
    /// and `ted_k`). The `size_diff` and running-remainder early exits only fire
    /// when the *minimum still-achievable* LB already exceeds `k`, so they never
    /// turn a within-`k` pair into an over-bound one.
    fn lblint_bounded_lb(&self, other: &UnifiedTreeIndex, k: i32) -> i32 {
        let bigger = self.tree_size.max(other.tree_size) as i32;

        // Size difference alone is a lower bound on the LB (hence on TED).
        let size_diff = self.tree_size.abs_diff(other.tree_size) as i32;
        if size_diff > k {
            return k + 1;
        }

        // Count the smaller multiset, probe with the larger, to keep the map small.
        let (small, large) = if self.labels.len() <= other.labels.len() {
            (self, other)
        } else {
            (other, self)
        };

        let mut counts: rustc_hash::FxHashMap<&str, i32> = rustc_hash::FxHashMap::default();
        for l in &small.labels {
            *counts.entry(l.as_str()).or_insert(0) += 1;
        }

        let mut intersection = 0i32;
        let mut remaining = large.labels.len() as i32;
        for l in &large.labels {
            remaining -= 1;
            if let Some(cnt) = counts.get_mut(l.as_str()) {
                if *cnt > 0 {
                    *cnt -= 1;
                    intersection += 1;
                }
            }
            // `bigger - (intersection + remaining)` is the smallest final LB still
            // achievable (every remaining node matching). If even that exceeds k,
            // the true LB certainly exceeds k → over-bound.
            if bigger - intersection - remaining > k {
                return k + 1;
            }
        }

        bigger - intersection
    }
}

// ---------------------------------------------------------------------------
// Pipeline
// ---------------------------------------------------------------------------

/// Combined label-intersection LB filter → TopDiff verification pipeline over a
/// single pre-indexed `UnifiedTreeIndex` column. Each argument's CBOR is
/// deserialized once; Stage 1 (cheap label-intersection lower bound) is computed
/// directly from the substrate's postorder label multisets, and only survivors
/// are interned into one shared dictionary, expanded into the TopDiff working
/// form, and run through Stage 2 (exact bounded TopDiff).
///
/// Returns the TopDiff distance when the pair passes both stages (`<= k`),
/// otherwise `k + 1` (over-bound), composable with `<= k` filters in SQL.
/// Sound: label intersection is a true lower bound on TED, so `LB > k ⇒ TED > k`.
#[pg_extern(immutable, parallel_safe, cost = 5000)]
fn lblint_topdiff_within(query: UnifiedTreeIndex, cand: UnifiedTreeIndex, k: i32) -> i32 {
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
    // Stage 1 — label-intersection lower bound, straight off the substrate. No
    // `expand()` yet: filter out if LB > k without building any working form.
    if query.lblint_bounded_lb(&cand, k) > k {
        return k + 1;
    }
    // Stage 2 — exact bounded TopDiff. One shared label dictionary across both
    // trees; discard the SED halves. `ted_k` already returns k+1 on over-bound.
    let mut dict = rustc_hash::FxHashMap::default();
    let (_q_sed, q_td) = query.expand(&mut dict);
    let (_c_sed, c_td) = cand.expand(&mut dict);
    ted_k(&q_td, &c_td, k)
}

// ===========================================================================
// Unit tests (run WITHOUT Postgres; #[pg_extern] fns are plain Rust callable).
// ===========================================================================

#[cfg(test)]
mod lblint_unit_tests {
    use super::*;
    use crate::parsing::parse_tree;
    use crate::types::{TreeArena, UnifiedTreeIndex};
    use std::ffi::CString;

    /// Spread of small trees (chains, bushy, deep) with varied label overlap and
    /// size differences. Copied from the lib.rs `tests` corpus.
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

    /// Exact (unbounded) reference label-intersection LB via the existing
    /// TreeArena filter. A large bound makes `bounded_label_intersection_distance`
    /// return the exact LB rather than the over-bound signal. On this corpus every
    /// tree has distinct intra-tree labels, so the set-based reference and my
    /// multiset stage-1 statistic coincide.
    fn ref_lb(a: &str, b: &str) -> i32 {
        crate::lb::label_intersection::bounded_label_intersection_distance(&ta(a), &ta(b), 1_000_000)
            as i32
    }

    /// LB differential: my substrate-based Stage-1 LB (computed unbounded via a
    /// huge k) must equal the existing reference for every corpus pair.
    #[test]
    fn stage1_lb_matches_reference() {
        for &a in TREES {
            for &b in TREES {
                let mine = uti(a).lblint_bounded_lb(&uti(b), 1_000_000);
                let reference = ref_lb(a, b);
                assert_eq!(
                    mine, reference,
                    "stage-1 LB {mine} != reference {reference} for ({a}, {b})"
                );
            }
        }
    }

    /// Bounded return semantics: for several bounds, my bounded LB's over-bound
    /// DECISION (`> k`) must agree with the reference's (`> k`), and whenever it
    /// reports within-bound it must return the exact LB.
    #[test]
    fn stage1_bounded_semantics_match_reference() {
        for &a in TREES {
            for &b in TREES {
                let exact = ref_lb(a, b);
                for &k in &[0i32, 1, 2, 3, 7, 50] {
                    let mine = uti(a).lblint_bounded_lb(&uti(b), k);
                    // Over-bound iff exact LB > k; my signal is exactly k+1.
                    if exact > k {
                        assert_eq!(
                            mine,
                            k + 1,
                            "expected over-bound k+1 for ({a},{b},k={k}); exact LB {exact}"
                        );
                    } else {
                        assert_eq!(
                            mine, exact,
                            "expected exact LB {exact} within bound for ({a},{b},k={k})"
                        );
                    }
                }
            }
        }
    }

    /// LB soundness: Stage-1 LB never exceeds the exact TED (else it could wrongly
    /// filter a true match). Large bound so nothing short-circuits.
    #[test]
    fn stage1_lb_is_sound() {
        for &a in TREES {
            for &b in TREES {
                let exact = crate::tree_ed(ta(a), ta(b));
                let lb = uti(a).lblint_bounded_lb(&uti(b), 1_000_000);
                assert!(
                    lb <= exact,
                    "label-intersection LB {lb} exceeds exact TED {exact} for ({a}, {b})"
                );
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
                    let got = super::lblint_topdiff_within(uti(a), uti(b), k);
                    assert_eq!(
                        got, expected,
                        "lblint_topdiff_within({a}, {b}, {k}) = {got}, expected {expected} (exact TED {exact})"
                    );
                }
            }
        }
    }

    /// Edge cases: empty×empty, empty×tree within/over budget, negative k,
    /// identical trees at k=0.
    #[test]
    fn edge_cases() {
        let empty = || UnifiedTreeIndex {
            labels: vec![],
            sizes: vec![],
            tree_size: 0,
        };
        // Two empty trees: TED 0.
        assert_eq!(super::lblint_topdiff_within(empty(), empty(), 3), 0);
        // Empty vs 3-node tree within budget: TED 3.
        assert_eq!(super::lblint_topdiff_within(empty(), uti("{a{b}{c}}"), 5), 3);
        // Empty vs 3-node tree, budget too small: k+1.
        assert_eq!(super::lblint_topdiff_within(empty(), uti("{a{b}{c}}"), 1), 2);
        // Negative k is always over-bound (returns k+1).
        assert_eq!(super::lblint_topdiff_within(uti("{a}"), uti("{a}"), -1), 0);
        // Identical trees at k=0: exact TED 0.
        assert_eq!(
            super::lblint_topdiff_within(uti("{a{b}{c}}"), uti("{a{b}{c}}"), 0),
            0
        );
    }

    /// Off-by-one probes: for a pair with exact TED d ≥ 1, k=d returns d (exact,
    /// within budget) and k=d−1 returns d (= k+1, over budget). Both yield d but
    /// via different code paths — a tight guard against a ±1 slip in either stage.
    #[test]
    fn off_by_one_probes() {
        for &(a, b) in &[
            ("{a{b}{c}}", "{a{b}{x}}"), // d = 1
            ("{a}", "{a{b}{c}}"),       // d = 2
            ("{x{y}{z}}", "{a{b}{c}}"), // fully disjoint labels
        ] {
            let d = crate::tree_ed(ta(a), ta(b));
            assert!(d >= 1, "corpus probe ({a},{b}) must have TED >= 1, got {d}");
            assert_eq!(
                super::lblint_topdiff_within(uti(a), uti(b), d),
                d,
                "k=d ({d}) should return exact d for ({a},{b})"
            );
            assert_eq!(
                super::lblint_topdiff_within(uti(a), uti(b), d - 1),
                d,
                "k=d-1 ({}) should return k+1 = d ({d}) for ({a},{b})",
                d - 1
            );
            if d >= 2 {
                assert_eq!(
                    super::lblint_topdiff_within(uti(a), uti(b), d - 2),
                    d - 1,
                    "k=d-2 ({}) should return k+1 = d-1 for ({a},{b})",
                    d - 2
                );
            }
        }
    }
}

// ===========================================================================
// Postgres SPI round-trip (booted by the orchestrator's full pg_test run only).
// ===========================================================================

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    /// Round-trip through the SQL boundary: bracket-notation input function +
    /// CBOR arg serialization + the full pipeline. Uniquely named (the `#[pg_test]`
    /// macro emits a global `pg_finfo_<fn>_wrapper` symbol) to avoid colliding with
    /// the other pipelines' round-trip tests.
    #[pg_test]
    fn lblint_sql_round_trip() {
        let same = Spi::get_one::<i32>(
            "SELECT lblint_topdiff_within('{a{b}{c}}'::unifiedtreeindex, '{a{b}{c}}'::unifiedtreeindex, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(same, 0);
        let one = Spi::get_one::<i32>(
            "SELECT lblint_topdiff_within('{a{b}{c}}'::unifiedtreeindex, '{a{b}{x}}'::unifiedtreeindex, 5)",
        )
        .unwrap()
        .unwrap();
        assert_eq!(one, 1);
    }
}
