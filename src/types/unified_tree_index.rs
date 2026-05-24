//! `UnifiedTreeIndex` — the single pre-indexed postorder substrate that expands
//! per call into both pipeline working forms (SED-struct-int + `TopDiffIndex`),
//! so a dataset tree is materialized once and reused across the SED-Struct LB
//! filter and TopDiff verification stages.
//!
//! See `docs/superpowers/plans/2026-05-24-unified-tree-index-pipeline.md` (Track B).

use pgrx::prelude::*;
use serde::{Deserialize, Serialize};

use crate::parsing::parse_tree;
use crate::types::tree_internals::id::NodeId;
use crate::TreeArena;

/// Postorder substrate: labels + subtree sizes (+ count). Postorder ids together
/// with subtree sizes uniquely determine the tree, so node depths, left-child
/// links, and the SED `sum`/`diff` annotations are all derived in the per-call
/// `expand` pass rather than stored — keeping the CBOR payload small.
#[derive(Debug, Clone, PartialEq, Eq, PostgresType, Serialize, Deserialize)]
#[inoutfuncs]
pub struct UnifiedTreeIndex {
    pub labels: Vec<String>, // postorder
    pub sizes: Vec<i32>,     // postorder subtree sizes
    pub tree_size: usize,
}

impl From<TreeArena> for UnifiedTreeIndex {
    fn from(tree: TreeArena) -> Self {
        let tree_size = tree.count();
        let mut labels = Vec::with_capacity(tree_size);
        let mut sizes = Vec::with_capacity(tree_size);

        if tree_size == 0 {
            return Self { labels, sizes, tree_size: 0 };
        }

        let root = tree.iter().next().expect("tree is non-empty");
        let root_id = tree.get_node_id(root).expect("root must be in arena");

        collect_postorder(root_id, &tree, &mut labels, &mut sizes);

        Self { labels, sizes, tree_size }
    }
}

/// Postorder DFS: visit children left→right recursively, then push self.
fn collect_postorder(
    nid: NodeId,
    tree: &TreeArena,
    labels: &mut Vec<String>,
    sizes: &mut Vec<i32>,
) -> i32 {
    let mut sz = 1i32;
    for cnid in nid.children(tree) {
        sz += collect_postorder(cnid, tree, labels, sizes);
    }
    let label = tree.get(nid).unwrap().get().clone();
    labels.push(label);
    sizes.push(sz);
    sz
}

impl InOutFuncs for UnifiedTreeIndex {
    fn input(input: &core::ffi::CStr) -> Self
    where
        Self: Sized,
    {
        Self::from(parse_tree(input).expect("failed to parse input tree"))
    }

    fn output(&self, buffer: &mut pgrx::StringInfo) {
        // Debug rendering: "label1,label2,...:size1,size2,..."
        buffer.push_str(&self.labels.join(","));
        buffer.push_str(":");
        let size_strs: Vec<String> = self.sizes.iter().map(|s| s.to_string()).collect();
        buffer.push_str(&size_strs.join(","));
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(s: &str) -> TreeArena {
        parse_tree(std::ffi::CString::new(s).unwrap().as_c_str()).unwrap()
    }

    // -----------------------------------------------------------------------
    // B1: UnifiedTreeIndex postorder substrate
    // -----------------------------------------------------------------------

    #[test]
    fn unified_from_tree_postorder() {
        let t = pt("{a{b}{c}}");
        let u = UnifiedTreeIndex::from(t);
        assert_eq!(u.labels, vec!["b", "c", "a"]);
        assert_eq!(u.sizes, vec![1, 1, 3]);
        assert_eq!(u.tree_size, 3);
    }
}
