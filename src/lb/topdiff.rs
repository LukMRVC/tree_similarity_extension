//! Rust port of the C++ Touzet KR-set bounded tree edit distance (TopDiff).
//!
//! Built test-first against the retained C++ oracle `cppffi::tree_topdiff_bounded`.
//! See `docs/superpowers/plans/2026-05-24-unified-tree-index-pipeline.md` (Track A).

/// Rust equivalent of C++ `node::TreeIndexTouzetKRSet`. All vectors are indexed
/// by left-to-right POSTORDER id (`0..tree_size`).
///
/// INVARIANTS (must hold for both the reference builder in this module and the
/// production `expand` in `crate::types::unified_tree_index` — differential
/// tested at the A/B seam):
///   * labels are interned to `i32` against a dict SHARED with the SED form
///   * `postl_to_size`: subtree node count; leaf == 1
///   * `postl_to_depth`: root depth == 0
///   * `postl_to_lch`: leftmost-child postorder id; LEAF == -1
///   * `list_kr`: postorder ids of keyroots (non-first children + root); order irrelevant
///   * `postl_to_kr_ancestor`: nearest keyroot ancestor postorder id
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopDiffIndex {
    pub tree_size: i32,
    pub postl_to_label_id: Vec<i32>,
    pub postl_to_size: Vec<i32>,
    pub postl_to_depth: Vec<i32>,
    pub postl_to_lch: Vec<i32>,
    pub postl_to_kr_ancestor: Vec<i32>,
    pub list_kr: Vec<i32>,
}

/// Bounded tree edit distance via the Touzet KR-set algorithm. Returns the exact
/// TED when it is `<= k`, otherwise `k + 1` (the over-bound convention, matching
/// `bounded_sed_struct_int` and the C++ oracle `tree_topdiff_bounded`).
///
/// STUB — implemented in Track A (plan task A5).
#[allow(dead_code, unused_variables)]
pub fn ted_k(t1: &TopDiffIndex, t2: &TopDiffIndex, k: i32) -> i32 {
    todo!("Track A task A5: Touzet KR-set ted_k port")
}
