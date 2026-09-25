//! SED-Struct LB filter → TopDiff verification pipeline, the default `Lb`.

use pgrx::prelude::*;

use crate::lb::sed::bounded_sed_struct_int;
use crate::lb::ted::topdiff::ted_k;
use crate::types::{Tree, UnifiedTreeIndex};

/// Combined SED-Struct LB filter → TopDiff verification pipeline. Both trees are
/// expanded into the SED and TopDiff working forms (labels are label hashes, so
/// no dictionary is built), then run through Stage 1 (cheap SED-Struct lower
/// bound) and — only on survivors — Stage 2 (exact bounded TopDiff).
///
/// Returns the TopDiff distance when the pair passes both stages (`<= k`),
/// otherwise `k + 1` (over-bound), composable with `<= k` filters in SQL.
/// Sound: SED-Struct is a true lower bound on TED, so `LB > k ⇒ TED > k`.
#[pg_extern(immutable, parallel_safe, cost = 5000)]
fn sed_topdiff_within(query: Tree, cand: Tree, k: i32) -> i32 {
    sed_struct_within(&query.to_unified(), &cand.to_unified(), k)
}

pub fn sed_struct_within(query: &UnifiedTreeIndex, cand: &UnifiedTreeIndex, k: i32) -> i32 {
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
    let (q_sed, q_td) = query.expand();
    let (c_sed, c_td) = cand.expand();
    // Stage 1 — SED-Struct lower bound (hashed labels). Filtered out if LB > k.
    if bounded_sed_struct_int(&q_sed, &c_sed, k_usize + 1) > k_usize {
        return k + 1;
    }
    // Stage 2 — exact bounded TopDiff; already returns k+1 on over-bound.
    ted_k(&q_td, &c_td, k)
}
