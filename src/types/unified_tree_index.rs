//! `UnifiedTreeIndex` — the single pre-indexed postorder substrate that expands
//! per call into both pipeline working forms (SED-struct-int + `TopDiffIndex`),
//! so a dataset tree is materialized once and reused across the SED-Struct LB
//! filter and TopDiff verification stages.
//!
//! See `docs/superpowers/plans/2026-05-24-unified-tree-index-pipeline.md` (Track B).

use crate::parsing::{label_hash, walk_bracket, BracketVisitor, LabelHash, TreeParseError};
use crate::lb::sed::{SEDStructIndexInt, TraversalCharacterInt};
use crate::lb::ted::topdiff::TopDiffIndex;
use crate::types::tree_internals::id::NodeId;
use crate::TreeArena;

/// Postorder substrate: label hashes + subtree sizes (+ count). Postorder ids
/// together with subtree sizes uniquely determine the tree, so node depths,
/// left-child links, and the SED `sum`/`diff` annotations are all derived in the
/// per-call `expand` pass rather than stored — keeping the index entries small.
///
/// Labels are stored as their [`label_hash`], computed once while parsing. The
/// hash is a global label id, so two trees can be compared without building a
/// shared label dictionary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnifiedTreeIndex {
    pub labels: Vec<LabelHash>, // postorder
    pub sizes: Vec<i32>,        // postorder subtree sizes
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
    labels: &mut Vec<LabelHash>,
    sizes: &mut Vec<i32>,
) -> i32 {
    let mut sz = 1i32;
    for cnid in nid.children(tree) {
        sz += collect_postorder(cnid, tree, labels, sizes);
    }
    labels.push(label_hash(tree.get(nid).unwrap().get().as_bytes()));
    sizes.push(sz);
    sz
}

/// Emits nodes in postorder as they close: a node's label hash and subtree size
/// are known once its closing brace is reached.
struct PostorderBuilder {
    /// Open nodes: (label hash, size of the subtree seen so far).
    open: Vec<(LabelHash, i32)>,
    labels: Vec<LabelHash>,
    sizes: Vec<i32>,
}

impl BracketVisitor for PostorderBuilder {
    fn open(&mut self, label: &[u8]) {
        self.open.push((label_hash(label), 1));
    }

    fn close(&mut self) {
        let Some((hash, size)) = self.open.pop() else {
            return;
        };
        self.labels.push(hash);
        self.sizes.push(size);
        if let Some(parent) = self.open.last_mut() {
            parent.1 += size;
        }
    }
}

impl UnifiedTreeIndex {
    /// Parse bracket notation straight into the postorder substrate, hashing each
    /// label as it is read — no `TreeArena` and no `String` per label.
    ///
    /// Tokenizes with the same `walk_bracket` as `parse_tree`, so the result
    /// equals `UnifiedTreeIndex::from(parse_tree(input)?)`, including for nodes
    /// left unclosed at the end of the input (closed implicitly here).
    pub fn parse(input: &core::ffi::CStr) -> Result<Self, TreeParseError> {
        let bytes = input.to_bytes();
        let mut builder = PostorderBuilder {
            open: Vec::new(),
            labels: Vec::with_capacity(bytes.len() / 3),
            sizes: Vec::with_capacity(bytes.len() / 3),
        };
        walk_bracket(bytes, &mut builder)?;
        while !builder.open.is_empty() {
            builder.close();
        }
        let tree_size = builder.labels.len();
        Ok(Self {
            labels: builder.labels,
            sizes: builder.sizes,
            tree_size,
        })
    }
}

// ============================================================================
// expand: reconstruct SED-struct-int form AND TopDiffIndex from postorder substrate
// ============================================================================

impl UnifiedTreeIndex {
    /// Expand the postorder substrate into the two pipeline working forms. Label
    /// ids are the stored label hashes, which are globally consistent, so no
    /// dictionary is shared between the two trees being compared.
    pub fn expand(&self) -> (SEDStructIndexInt, TopDiffIndex) {
        let n = self.tree_size;
        assert_eq!(self.labels.len(), n);
        assert_eq!(self.sizes.len(), n);

        // Empty tree: return empty forms rather than underflowing the topology
        // reconstruction below (`n - 1`). The pipeline's size-diff gate lets an
        // empty/empty pair reach here.
        if n == 0 {
            return (
                SEDStructIndexInt {
                    first_traversal: Vec::new(),
                    second_traversal: Vec::new(),
                    tree_size: 0,
                },
                TopDiffIndex {
                    tree_size: 0,
                    postl_to_label_id: Vec::new(),
                    postl_to_size: Vec::new(),
                    postl_to_depth: Vec::new(),
                    postl_to_lch: Vec::new(),
                    postl_to_kr_ancestor: Vec::new(),
                    list_kr: Vec::new(),
                },
            );
        }

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

        for (i, child_slot) in children.iter_mut().enumerate() {
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
            *child_slot = child_ids;
            stack.push(i);
        }

        // Compute depths top-down (root = n-1, depth 0).
        let mut depth: Vec<i32> = vec![0; n];
        // Collect preorder sequence here (reused for both SED and TopDiff).
        let mut preorder_nodes: Vec<usize> = Vec::with_capacity(n);
        {
            let mut dfs_stack: Vec<(usize, i32)> = vec![(n - 1, 0)];
            while let Some((node, d)) = dfs_stack.pop() {
                depth[node] = d;
                preorder_nodes.push(node);
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
        // Label ids are the stored hashes (see `label_hash`).
        // ------------------------------------------------------------------

        let label_ids = &self.labels;

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
        // Step 3: Build TopDiffIndex.
        //
        // INVARIANTS (from topdiff.rs):
        //   * labels are the global label hashes (see `label_hash`)
        //   * postl_to_size: subtree node count; leaf == 1
        //   * postl_to_depth: root depth == 0
        //   * postl_to_lch: leftmost-child postorder id; LEAF == -1
        //   * list_kr: non-first children + root (order irrelevant)
        //   * postl_to_kr_ancestor: nearest keyroot ancestor postorder id
        // ------------------------------------------------------------------

        // postl_to_size and postl_to_depth are directly available.
        let postl_to_label_id = self.labels.clone();
        let postl_to_size: Vec<i32> = self.sizes.clone();
        let postl_to_depth = depth.clone();

        // postl_to_lch: for each node, postorder id of its leftmost child (or -1).
        let postl_to_lch: Vec<i32> = (0..n)
            .map(|i| children[i].first().map(|&c| c as i32).unwrap_or(-1))
            .collect();

        // is_keyroot: root + non-first children.
        let mut is_keyroot: Vec<bool> = vec![false; n];
        is_keyroot[n - 1] = true;
        for kids in &children {
            for &c in kids.iter().skip(1) {
                is_keyroot[c] = true;
            }
        }

        // list_kr: non-first children + root.
        let mut list_kr: Vec<i32> = vec![(n - 1) as i32]; // root
        for kids in &children {
            for &c in kids.iter().skip(1) {
                list_kr.push(c as i32);
            }
        }

        // postl_to_kr_ancestor: nearest keyroot ancestor (including self).
        // Process top-down (preorder) so parent's kr_ancestor is ready first.
        let mut postl_to_kr_ancestor: Vec<i32> = vec![-1; n];
        for &node in &preorder_nodes {
            if is_keyroot[node] {
                postl_to_kr_ancestor[node] = node as i32;
            } else {
                let p = parent[node];
                if p >= 0 {
                    postl_to_kr_ancestor[node] = postl_to_kr_ancestor[p as usize];
                }
            }
        }

        let topdiff = TopDiffIndex {
            tree_size: n as i32,
            postl_to_label_id,
            postl_to_size,
            postl_to_depth,
            postl_to_lch,
            postl_to_kr_ancestor,
            list_kr,
        };

        (sed_index, topdiff)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lb::sed::bounded_sed_struct_int;
    use crate::lb::ted::topdiff::TopDiffIndex;
    use crate::parsing::parse_tree;
    use std::ffi::CString;

    fn pt(s: &str) -> TreeArena {
        parse_tree(CString::new(s).unwrap().as_c_str()).unwrap()
    }

    fn h(label: &str) -> LabelHash {
        label_hash(label.as_bytes())
    }

    // -----------------------------------------------------------------------
    // B1: UnifiedTreeIndex postorder substrate
    // -----------------------------------------------------------------------

    #[test]
    fn unified_from_tree_postorder() {
        let t = pt("{a{b}{c}}");
        let u = UnifiedTreeIndex::from(t);
        assert_eq!(u.labels, vec![h("b"), h("c"), h("a")]);
        assert_eq!(u.sizes, vec![1, 1, 3]);
        assert_eq!(u.tree_size, 3);
    }

    /// The hashing parser must produce exactly what the reference path
    /// (`parse_tree` → `From<TreeArena>`) does, including escaped braces, text
    /// after a closing brace (ignored), and nodes left unclosed at the end.
    #[test]
    fn parse_matches_from_tree_arena() {
        let cases = [
            "{a}",
            "{a{b}}",
            "{a{b}{c}}",
            "{a{b{e}}{c}}",
            "{r{a{b}{c}}{d{e}}}",
            "{1{2}{3{4}}}",
            "{a b{c d}{}}",
            r"{a\{b}",
            r"{a\}b}",
            r"{a\\}b}",
            r"{a{b\}}{c}}",
            "{a{b}c}",
            "{a{b}{c}",
            "{a{b{c}",
            "{x{y}{z}}trailing",
        ];
        for s in cases {
            let c = CString::new(s).unwrap();
            let parsed = UnifiedTreeIndex::parse(c.as_c_str()).unwrap();
            let reference = UnifiedTreeIndex::from(parse_tree(c.as_c_str()).unwrap());
            assert_eq!(parsed, reference, "parse vs From<TreeArena> mismatch for {s}");
        }
    }

    /// Both parsers reject the same malformed inputs.
    #[test]
    fn parse_rejects_what_parse_tree_rejects() {
        for s in [r"\{a}", "{a}}", "{a}{b}", "{a}{"] {
            let c = CString::new(s).unwrap();
            assert!(parse_tree(c.as_c_str()).is_err(), "parse_tree accepted {s}");
            assert!(UnifiedTreeIndex::parse(c.as_c_str()).is_err(), "parse accepted {s}");
        }
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
        let (sed1, _td1) = u1.expand();
        let (sed2, _td2) = u2.expand();
        assert_eq!(sed1.first_traversal, ref1.first_traversal);
        assert_eq!(sed1.second_traversal, ref1.second_traversal);
        assert_eq!(sed2.first_traversal, ref2.first_traversal);
        assert_eq!(sed2.second_traversal, ref2.second_traversal);
        assert_eq!(
            bounded_sed_struct_int(&sed1, &sed2, 10),
            bounded_sed_struct_int(&ref1, &ref2, 10)
        );
    }

    // -----------------------------------------------------------------------
    // B3: expand TopDiff half matches reference builder (the contract seam).
    //
    // The reference builder is duplicated here for differential testing; it
    // will be reconciled with Track A's TopDiffIndex::from_tree at integration.
    // -----------------------------------------------------------------------

    /// Reference TopDiffIndex builder direct from a TreeArena, honoring all
    /// TopDiffIndex invariants. Labels are hashed with `label_hash`.
    fn reference_topdiff_index(tree: &TreeArena) -> TopDiffIndex {
        let n = tree.count();

        let root = tree.iter().next().expect("tree non-empty");
        let root_id = tree.get_node_id(root).unwrap();

        // Postorder numbering.
        let mut postorder: Vec<NodeId> = Vec::with_capacity(n);
        collect_postorder_ids(root_id, tree, &mut postorder);

        use std::collections::HashMap;
        let mut nid_to_postl: HashMap<NodeId, usize> = HashMap::with_capacity(n);
        for (i, &nid) in postorder.iter().enumerate() {
            nid_to_postl.insert(nid, i);
        }

        // Preorder ids (needed for depth and label interning order).
        let mut preorder_ids: Vec<NodeId> = Vec::with_capacity(n);
        collect_preorder_ids(root_id, tree, &mut preorder_ids);

        // Depth top-down (root = 0).
        let mut postl_to_depth: Vec<i32> = vec![0; n];
        {
            let mut dfs: Vec<(NodeId, i32)> = vec![(root_id, 0)];
            while let Some((nid, d)) = dfs.pop() {
                postl_to_depth[nid_to_postl[&nid]] = d;
                let node = tree.get(nid).unwrap();
                let mut ch: Vec<NodeId> = Vec::new();
                let mut cur = node.first_child;
                while let Some(c) = cur {
                    ch.push(c);
                    cur = tree.get(c).unwrap().next_sibling;
                }
                for c in ch.into_iter().rev() {
                    dfs.push((c, d + 1));
                }
            }
        }

        // Label ids are the label hashes.
        let mut postl_to_label_id: Vec<LabelHash> = vec![0; n];
        for &nid in &preorder_ids {
            let label = tree.get(nid).unwrap().get();
            let id = label_hash(label.as_bytes());
            postl_to_label_id[nid_to_postl[&nid]] = id;
        }

        let mut postl_to_size: Vec<i32> = Vec::with_capacity(n);
        let mut postl_to_lch: Vec<i32> = Vec::with_capacity(n);

        for &nid in &postorder {
            // descendants() includes self → subtree size.
            let subtree_sz = nid.descendants(tree).count() as i32;
            postl_to_size.push(subtree_sz);

            let fc = tree.get(nid).unwrap().first_child;
            let lch = fc.map(|c| nid_to_postl[&c] as i32).unwrap_or(-1);
            postl_to_lch.push(lch);
        }

        // is_keyroot: root + non-first children.
        let mut is_keyroot: Vec<bool> = vec![false; n];
        is_keyroot[n - 1] = true;
        for &nid in &postorder {
            let node = tree.get(nid).unwrap();
            let mut first = true;
            let mut cur = node.first_child;
            while let Some(c) = cur {
                if !first {
                    is_keyroot[nid_to_postl[&c]] = true;
                }
                first = false;
                cur = tree.get(c).unwrap().next_sibling;
            }
        }

        // list_kr: non-first children + root.
        let mut list_kr: Vec<i32> = vec![(n - 1) as i32];
        for &nid in &postorder {
            let node = tree.get(nid).unwrap();
            let mut first = true;
            let mut cur = node.first_child;
            while let Some(c) = cur {
                if !first {
                    list_kr.push(nid_to_postl[&c] as i32);
                }
                first = false;
                cur = tree.get(c).unwrap().next_sibling;
            }
        }

        // postl_to_kr_ancestor in preorder.
        let mut postl_to_kr_ancestor: Vec<i32> = vec![-1; n];
        for &nid in &preorder_ids {
            let postl = nid_to_postl[&nid];
            if is_keyroot[postl] {
                postl_to_kr_ancestor[postl] = postl as i32;
            } else {
                let par = tree.get(nid).unwrap().parent;
                if let Some(par_id) = par {
                    let par_postl = nid_to_postl[&par_id];
                    postl_to_kr_ancestor[postl] = postl_to_kr_ancestor[par_postl];
                }
            }
        }

        TopDiffIndex {
            tree_size: n as i32,
            postl_to_label_id,
            postl_to_size,
            postl_to_depth,
            postl_to_lch,
            postl_to_kr_ancestor,
            list_kr,
        }
    }

    fn collect_postorder_ids(nid: NodeId, tree: &TreeArena, out: &mut Vec<NodeId>) {
        for c in nid.children(tree) {
            collect_postorder_ids(c, tree, out);
        }
        out.push(nid);
    }

    fn collect_preorder_ids(nid: NodeId, tree: &TreeArena, out: &mut Vec<NodeId>) {
        out.push(nid);
        for c in nid.children(tree) {
            collect_preorder_ids(c, tree, out);
        }
    }

    #[test]
    fn expand_topdiff_matches_reference() {
        for s in &["{a}", "{a{b}{c}}", "{a{b{d}}{c}}"] {
            let t = pt(s);

            let ref_td = reference_topdiff_index(&t);

            let u = UnifiedTreeIndex::from(t);
            let (_sed, td) = u.expand();

            assert_eq!(td.tree_size, ref_td.tree_size, "tree_size mismatch for {}", s);
            assert_eq!(td.postl_to_label_id, ref_td.postl_to_label_id, "label_id mismatch for {}", s);
            assert_eq!(td.postl_to_size, ref_td.postl_to_size, "size mismatch for {}", s);
            assert_eq!(td.postl_to_depth, ref_td.postl_to_depth, "depth mismatch for {}", s);
            assert_eq!(td.postl_to_lch, ref_td.postl_to_lch, "lch mismatch for {}", s);
            assert_eq!(td.postl_to_kr_ancestor, ref_td.postl_to_kr_ancestor, "kr_ancestor mismatch for {}", s);

            // list_kr: order irrelevant; compare as sorted sets.
            let mut got = td.list_kr.clone();
            let mut exp = ref_td.list_kr.clone();
            got.sort();
            exp.sort();
            assert_eq!(got, exp, "list_kr mismatch for {}", s);
        }
    }
}
