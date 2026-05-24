//! `UnifiedTreeIndex` — the single pre-indexed postorder substrate that expands
//! per call into both pipeline working forms (SED-struct-int + `TopDiffIndex`),
//! so a dataset tree is materialized once and reused across the SED-Struct LB
//! filter and TopDiff verification stages.
//!
//! See `docs/superpowers/plans/2026-05-24-unified-tree-index-pipeline.md` (Track B).

use pgrx::prelude::*;
use serde::{Deserialize, Serialize};

/// Postorder substrate: labels + subtree sizes (+ count). Postorder ids together
/// with subtree sizes uniquely determine the tree, so node depths, left-child
/// links, and the SED `sum`/`diff` annotations are all derived in the per-call
/// `expand` pass rather than stored — keeping the CBOR payload small.
///
/// STUB — `From<TreeArena>`, custom `InOutFuncs` (bracket-notation input), and
/// the `expand` routine are added in Track B (plan tasks B1–B3).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, PostgresType, Serialize, Deserialize)]
pub struct UnifiedTreeIndex {
    pub labels: Vec<String>, // postorder
    pub sizes: Vec<i32>,     // postorder subtree sizes
    pub tree_size: usize,
}
