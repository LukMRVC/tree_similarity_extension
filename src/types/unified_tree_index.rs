//! `UnifiedTreeIndex` — the single pre-indexed postorder substrate that expands
//! per call into both pipeline working forms (SED-struct-int + `TopDiffIndex`),
//! so a dataset tree is materialized once and reused across the SED-Struct LB
//! filter and TopDiff verification stages.
//!
//! See `docs/superpowers/plans/2026-05-24-unified-tree-index-pipeline.md` (Track B).

use pgrx::prelude::*;
use serde::{Deserialize, Serialize};

use crate::parsing::parse_tree;
use crate::lb::sed::{LabelDict, SEDStructIndexInt, TraversalCharacterInt};
use crate::lb::topdiff::TopDiffIndex;
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
// expand: reconstruct SED-struct-int form AND TopDiffIndex from postorder substrate
// ============================================================================

impl UnifiedTreeIndex {
    /// Expand the postorder substrate into the two pipeline working forms,
    /// interning all labels into the shared `dict` (cross-tree label-id
    /// consistency, exactly like `build_sed_struct_indices_int` sharing one dict).
    pub fn expand(&self, dict: &mut LabelDict) -> (SEDStructIndexInt, TopDiffIndex) {
        let n = self.tree_size;
        assert_eq!(self.labels.len(), n);
        assert_eq!(self.sizes.len(), n);

        // ------------------------------------------------------------------
        // Step 1: Reconstruct tree topology from postorder {labels, sizes}.
        //
        // For each postorder node i, its subtree has `sizes[i]` nodes. Using
        // a stack we can identify direct children: they are the stack entries
        // whose sizes collectively sum to `sizes[i] - 1`.
        // ------------------------------------------------------------------

        let mut parent: Vec<i32> = vec![-1; n];
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];

        let mut stack: Vec<usize> = Vec::with_capacity(n);

        for i in 0..n {
            let sz = self.sizes[i] as usize;
            let mut remaining = sz - 1;
            let mut child_ids: Vec<usize> = Vec::new();
            while remaining > 0 {
                let c = *stack.last().expect("stack underflow during topology reconstruction");
                let c_sz = self.sizes[c] as usize;
                stack.pop();
                child_ids.push(c);
                remaining -= c_sz;
            }
            // child_ids is right→left (last child popped first); reverse for left→right.
            child_ids.reverse();

            for &c in &child_ids {
                parent[c] = i as i32;
            }
            children[i] = child_ids;
            stack.push(i);
        }

        // Compute depths top-down (root = n-1, depth 0).
        let mut depth: Vec<i32> = vec![0; n];
        {
            let mut dfs_stack: Vec<(usize, i32)> = vec![(n - 1, 0)];
            while let Some((node, d)) = dfs_stack.pop() {
                depth[node] = d;
                for &c in children[node].iter().rev() {
                    dfs_stack.push((c, d + 1));
                }
            }
        }

        // ------------------------------------------------------------------
        // Step 2: Build SED-struct-int form.
        //
        // Mirrors traverse_with_info_int exactly:
        //   postorder_id (1-based) for node i = i + 1
        //   preceding = (i+1).saturating_sub(sizes[i])
        //   following = tree_size.saturating_sub((i+1) + depth[i])
        //   descendant = sizes[i] - 1
        //   ancestor = depth[i]
        //   pre.sum  = following + descendant
        //   pre.diff = descendant - following
        //   rev_post.sum  = preceding + ancestor
        //   rev_post.diff = ancestor - preceding
        //
        // CRITICAL: intern labels in preorder visit order (same as
        // traverse_with_info_int, which descends into each node before children).
        // ------------------------------------------------------------------

        // Collect preorder sequence (root first, left→right children).
        let mut preorder_nodes: Vec<usize> = Vec::with_capacity(n);
        {
            let mut dfs_stack: Vec<usize> = vec![n - 1];
            while let Some(node) = dfs_stack.pop() {
                preorder_nodes.push(node);
                for &c in children[node].iter().rev() {
                    dfs_stack.push(c);
                }
            }
        }

        // Intern labels in preorder order.
        let mut label_ids: Vec<i32> = vec![0; n];
        for &node in &preorder_nodes {
            label_ids[node] = intern_local(dict, &self.labels[node]);
        }

        let mut first_traversal: Vec<TraversalCharacterInt> = Vec::with_capacity(n);
        let mut second_traversal_rev: Vec<TraversalCharacterInt> = Vec::with_capacity(n);

        for &node in &preorder_nodes {
            let sz = self.sizes[node] as usize;
            let d = depth[node] as usize;
            let pid = node + 1; // postorder_id after increment

            let preceding = pid.saturating_sub(sz);
            let following = n.saturating_sub(pid + d);
            let descendant = sz as i32 - 1;
            let ancestor = d as i32;

            first_traversal.push(TraversalCharacterInt {
                label: label_ids[node],
                sum: following as i32 + descendant,
                diff: descendant - following as i32,
            });
            second_traversal_rev.push(TraversalCharacterInt {
                label: label_ids[node],
                sum: preceding as i32 + ancestor,
                diff: ancestor - preceding as i32,
            });
        }

        // traverse_with_info_int fills reversed_postorder in preorder order
        // then calls .reverse() — we do the same.
        second_traversal_rev.reverse();

        let sed_index = SEDStructIndexInt {
            first_traversal,
            second_traversal: second_traversal_rev,
            tree_size: n,
        };

        // ------------------------------------------------------------------
        // Step 3: Build TopDiffIndex (stub — TopDiff half implemented in B3).
        // ------------------------------------------------------------------

        let topdiff = build_topdiff_stub(n);

        (sed_index, topdiff)
    }
}

/// Temporary stub for the TopDiff half (replaced in B3).
fn build_topdiff_stub(n: usize) -> TopDiffIndex {
    TopDiffIndex {
        tree_size: n as i32,
        postl_to_label_id: vec![0; n],
        postl_to_size: vec![0; n],
        postl_to_depth: vec![0; n],
        postl_to_lch: vec![-1; n],
        postl_to_kr_ancestor: vec![-1; n],
        list_kr: Vec::new(),
    }
}

/// Inline intern: lookup or insert into dict, returning the i32 id.
#[inline]
fn intern_local(dict: &mut LabelDict, label: &str) -> i32 {
    if let Some(&id) = dict.get(label) {
        return id;
    }
    let id = dict.len() as i32;
    dict.insert(label.to_owned(), id);
    id
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lb::sed::bounded_sed_struct_int;
    use rustc_hash::FxHashMap;

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

    // -----------------------------------------------------------------------
    // B2: expand SED half matches existing build_sed_struct_indices_int
    // -----------------------------------------------------------------------

    #[test]
    fn expand_sed_form_matches_existing() {
        let t1 = pt("{a{b}{c}}");
        let t2 = pt("{a{d}{c}}");
        let (ref1, ref2) = crate::lb::sed::build_sed_struct_indices_int(&t1, &t2);
        let u1 = UnifiedTreeIndex::from(t1);
        let u2 = UnifiedTreeIndex::from(t2);
        let mut dict = FxHashMap::default();
        let (sed1, _td1) = u1.expand(&mut dict);
        let (sed2, _td2) = u2.expand(&mut dict);
        assert_eq!(sed1.first_traversal, ref1.first_traversal);
        assert_eq!(sed1.second_traversal, ref1.second_traversal);
        assert_eq!(sed2.first_traversal, ref2.first_traversal);
        assert_eq!(sed2.second_traversal, ref2.second_traversal);
        assert_eq!(
            bounded_sed_struct_int(&sed1, &sed2, 10),
            bounded_sed_struct_int(&ref1, &ref2, 10)
        );
    }
}
