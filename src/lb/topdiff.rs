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

/// A specialised matrix where only the elements on a diagonal band matter.
/// Port of the C++ `data_structures::BandMatrix<double>` (`include/matrix.h`).
///
/// The backing store is a flat `rows * (2 * band_width + 1)` vector. Accessing
/// `(row, col)` translates the column to `col + band_width - row` (the diagonal
/// shift that reduces memory from `O(n^2)` to `O(n*k)`). Callers must ensure all
/// accessed cells are within the band, exactly as in the C++ version.
#[derive(Debug, Clone)]
pub struct BandMatrix {
    columns: usize,
    band_width: usize,
    data: Vec<f64>,
}

impl BandMatrix {
    /// Mirrors `BandMatrix(rows, band_width)` followed by `fill_with(fill)`.
    /// Backing matrix has `2 * band_width + 1` columns.
    pub fn new(rows: usize, band_width: usize, fill: f64) -> Self {
        let columns = 2 * band_width + 1;
        Self {
            columns,
            band_width,
            data: vec![fill; rows * columns],
        }
    }

    #[inline]
    fn translate(&self, row: usize, col: usize) -> usize {
        // C++: Matrix::at(row, col + band_width_ - row). The column translation
        // can transiently underflow in usize, so compute in isize then index.
        let translated = col as isize + self.band_width as isize - row as isize;
        row * self.columns + translated as usize
    }

    /// `BandMatrix::at(row, col)` for writing.
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        let idx = self.translate(row, col);
        self.data[idx] = value;
    }

    /// Mutable reference equivalent to C++ `at(row, col)`.
    #[inline]
    pub fn at(&mut self, row: usize, col: usize) -> &mut f64 {
        let idx = self.translate(row, col);
        &mut self.data[idx]
    }

    /// `BandMatrix::read_at(row, col)` — const read.
    #[inline]
    pub fn read_at(&self, row: usize, col: usize) -> f64 {
        let idx = self.translate(row, col);
        self.data[idx]
    }
}

impl TopDiffIndex {
    /// Reference builder. Ports `node::index_tree` / `index_tree_recursion` for
    /// the `TreeIndexTouzetKRSet` subset of indices, plus `fill_kr_ancestors`.
    ///
    /// Labels are interned into `dict` (get-or-insert: id = existing or
    /// `dict.len()`), matching `sed::intern`, so the resulting label ids are
    /// shared with the SED forms when the same dict is threaded through.
    pub fn from_tree(tree: &crate::types::TreeArena, dict: &mut crate::lb::sed::LabelDict) -> Self {
        let tree_size = tree.count();

        let mut postl_to_label_id = vec![0i32; tree_size];
        let mut postl_to_size = vec![0i32; tree_size];
        let mut postl_to_depth = vec![0i32; tree_size];
        let mut postl_to_lch = vec![-1i32; tree_size];
        let mut list_kr: Vec<i32> = Vec::new();

        // Running postorder id; incremented after a node and all its children
        // are processed (mirrors `start_postorder`).
        let mut next_postorder: i32 = 0;

        if let Some(root) = tree.get_root_id() {
            Self::recurse(
                root,
                tree,
                dict,
                0, // start_depth: root depth == 0
                &mut next_postorder,
                &mut postl_to_label_id,
                &mut postl_to_size,
                &mut postl_to_depth,
                &mut postl_to_lch,
                &mut list_kr,
            );
            // Root is appended to the keyroot list last (C++: list_kr_.push_back(start_postorder - 1)).
            list_kr.push(next_postorder - 1);
        }

        // fill_kr_ancestors: walk the left-path (lch chain) from each keyroot.
        let mut postl_to_kr_ancestor = vec![0i32; tree_size];
        for &i in &list_kr {
            let mut l = i;
            while l >= 0 {
                postl_to_kr_ancestor[l as usize] = i;
                l = postl_to_lch[l as usize];
            }
        }

        Self {
            tree_size: tree_size as i32,
            postl_to_label_id,
            postl_to_size,
            postl_to_depth,
            postl_to_lch,
            postl_to_kr_ancestor,
            list_kr,
        }
    }

    /// Postorder DFS. Returns the subtree size rooted at `nid` and assigns this
    /// node's postorder id, label, size, depth, lch; pushes non-first children
    /// to `list_kr`.
    #[allow(clippy::too_many_arguments)]
    fn recurse(
        nid: crate::types::tree_internals::id::NodeId,
        tree: &crate::types::TreeArena,
        dict: &mut crate::lb::sed::LabelDict,
        depth: i32,
        next_postorder: &mut i32,
        postl_to_label_id: &mut [i32],
        postl_to_size: &mut [i32],
        postl_to_depth: &mut [i32],
        postl_to_lch: &mut [i32],
        list_kr: &mut Vec<i32>,
    ) -> i32 {
        let mut desc_sum = 0i32;
        let mut first_child_postorder: i32 = -1;

        let mut is_first = true;
        for cnid in nid.children(tree) {
            let child_size = Self::recurse(
                cnid,
                tree,
                dict,
                depth + 1,
                next_postorder,
                postl_to_label_id,
                postl_to_size,
                postl_to_depth,
                postl_to_lch,
                list_kr,
            );
            desc_sum += child_size;
            // The child's postorder id is `*next_postorder - 1` after its recursion.
            let child_postorder = *next_postorder - 1;
            if is_first {
                first_child_postorder = child_postorder;
                is_first = false;
            } else {
                // Non-first children are keyroots.
                list_kr.push(child_postorder);
            }
        }

        // Now *next_postorder holds this node's postorder id.
        let this_postorder = *next_postorder as usize;

        let label = tree.get(nid).unwrap().get();
        // Inline intern (sed::intern is private): get-or-insert.
        let label_id = if let Some(&id) = dict.get(label) {
            id
        } else {
            let id = dict.len() as i32;
            dict.insert(label.to_owned(), id);
            id
        };

        postl_to_label_id[this_postorder] = label_id;
        postl_to_size[this_postorder] = desc_sum + 1;
        postl_to_depth[this_postorder] = depth;
        postl_to_lch[this_postorder] = first_child_postorder;

        *next_postorder += 1;
        desc_sum + 1
    }
}

/// Remaining error budget for the subtree pair `(x, y)`.
/// Port of `TEDAlgorithmTouzet::e_budget` (`ted_algorithm_touzet.h:285`).
///
/// `e(x,y) = k - |(|T1|-(x+1)-depth(x)) - (|T2|-(y+1)-depth(y))|`
///            `- |depth(x)-depth(y)| - |((x+1)-|T1_x|) - ((y+1)-|T2_y|)|`
///
/// May return a NEGATIVE value — callers must not clamp (matches the C++ note).
pub fn e_budget(t1: &TopDiffIndex, t2: &TopDiffIndex, x: i32, y: i32, k: i32) -> i32 {
    let x_size = t1.postl_to_size[x as usize];
    let y_size = t2.postl_to_size[y as usize];
    let dx = t1.postl_to_depth[x as usize];
    let dy = t2.postl_to_depth[y as usize];
    let lower_bound = ((t1.tree_size - (x + 1) - dx) - (t2.tree_size - (y + 1) - dy)).abs()
        + (dx - dy).abs()
        + (((x + 1) - x_size) - ((y + 1) - y_size)).abs();
    k - lower_bound
}

/// Whether subtrees `T1_x` and `T2_y` are k-relevant.
/// Port of `TEDAlgorithmTouzet::k_relevant` (`ted_algorithm_touzet.h:315`).
///
/// True iff `|(|T1|-(x+1)-depth(x)) - (|T2|-(y+1)-depth(y))| + |depth(x)-depth(y)|`
///          `+ ||T1_x|-|T2_y|| + |((x+1)-|T1_x|) - ((y+1)-|T2_y|)| <= k`.
pub fn k_relevant(t1: &TopDiffIndex, t2: &TopDiffIndex, x: i32, y: i32, k: i32) -> bool {
    let x_size = t1.postl_to_size[x as usize];
    let y_size = t2.postl_to_size[y as usize];
    let dx = t1.postl_to_depth[x as usize];
    let dy = t2.postl_to_depth[y as usize];
    let lower_bound = ((t1.tree_size - (x + 1) - dx) - (t2.tree_size - (y + 1) - dy)).abs()
        + (dx - dy).abs()
        + (x_size - y_size).abs()
        + (((x + 1) - x_size) - ((y + 1) - y_size)).abs();
    lower_bound <= k
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lb::sed::LabelDict;
    use crate::parsing::parse_tree;
    use crate::types::TreeArena;

    const INF: f64 = f64::INFINITY;

    fn pt(s: &str) -> TreeArena {
        parse_tree(std::ffi::CString::new(s).unwrap().as_c_str()).unwrap()
    }

    /// Build a TopDiffIndex from a bracket string with a fresh dict.
    fn build(s: &str) -> TopDiffIndex {
        let mut dict = LabelDict::default();
        TopDiffIndex::from_tree(&pt(s), &mut dict)
    }

    fn sorted(v: &[i32]) -> Vec<i32> {
        let mut v = v.to_vec();
        v.sort_unstable();
        v
    }

    #[test]
    fn from_tree_single_node() {
        let idx = build("{a}");
        assert_eq!(idx.tree_size, 1);
        assert_eq!(idx.postl_to_size, vec![1]);
        assert_eq!(idx.postl_to_depth, vec![0]);
        assert_eq!(idx.postl_to_lch, vec![-1]);
        assert_eq!(idx.list_kr, vec![0]);
        assert_eq!(idx.postl_to_kr_ancestor, vec![0]);
    }

    #[test]
    fn from_tree_two_leaves() {
        // {a{b}{c}} -> postorder: b=0, c=1, a=2
        let idx = build("{a{b}{c}}");
        assert_eq!(idx.tree_size, 3);
        assert_eq!(idx.postl_to_size, vec![1, 1, 3]);
        assert_eq!(idx.postl_to_depth, vec![1, 1, 0]);
        assert_eq!(idx.postl_to_lch, vec![-1, -1, 0]);
        // keyroots: non-first child c=1, root a=2
        assert_eq!(sorted(&idx.list_kr), vec![1, 2]);
        assert_eq!(idx.postl_to_kr_ancestor, vec![2, 1, 2]);
    }

    #[test]
    fn from_tree_nested() {
        // {a{b{d}}{c}} -> postorder: d=0, b=1, c=2, a=3
        let idx = build("{a{b{d}}{c}}");
        assert_eq!(idx.tree_size, 4);
        assert_eq!(idx.postl_to_size, vec![1, 2, 1, 4]);
        assert_eq!(idx.postl_to_depth, vec![2, 1, 1, 0]);
        assert_eq!(idx.postl_to_lch, vec![-1, 0, -1, 1]);
        // keyroots: c=2 (non-first child of a), root a=3
        assert_eq!(sorted(&idx.list_kr), vec![2, 3]);
        assert_eq!(idx.postl_to_kr_ancestor, vec![3, 3, 2, 3]);
    }

    #[test]
    fn e_budget_and_k_relevant() {
        // t1 = {a{b}{c}}: size [1,1,3], depth [1,1,0]
        let t1 = build("{a{b}{c}}");
        // Root vs root, identical tree, k=5: lower bound 0, full budget.
        assert_eq!(e_budget(&t1, &t1, 2, 2, 5), 5);
        assert!(k_relevant(&t1, &t1, 2, 2, 5));

        // t2 = {a{b{d}}{c}}: size [1,2,1,4], depth [2,1,1,0]
        let t2 = build("{a{b{d}}{c}}");
        // x=0 (leaf b in t1: size 1, depth 1), y=3 (root in t2: size 4, depth 0), k=0.
        // term1 = |(3-1-1)-(4-4-0)| = |1-0| = 1
        // term2 = |1-0| = 1
        // term3 = |((1)-1)-((4)-4)| = 0
        // e_budget lower bound = 2 -> e_budget = 0 - 2 = -2 (stays negative).
        assert_eq!(e_budget(&t1, &t2, 0, 3, 0), -2);
        assert!(e_budget(&t1, &t2, 0, 3, 0) < 0);
        // k_relevant adds ||T1_x|-|T2_y|| = |1-4| = 3 -> lb = 4 > 0 -> false.
        assert!(!k_relevant(&t1, &t2, 0, 3, 0));
    }

    #[test]
    fn band_matrix_translate_and_fill() {
        let mut m = BandMatrix::new(5, 4, INF);
        // Untouched cell reads back the fill value.
        assert_eq!(m.read_at(2, 3), INF);
        // After a write, the same cell reads back the written value.
        m.set(2, 3, 7.0);
        assert_eq!(m.read_at(2, 3), 7.0);
        // The mutable accessor maps to the same backing cell.
        assert_eq!(*m.at(2, 3), 7.0);

        // band_width = 0 edge: single-column backing store, 1 row.
        let mut z = BandMatrix::new(1, 0, INF);
        assert_eq!(z.read_at(0, 0), INF);
        z.set(0, 0, 3.5);
        assert_eq!(z.read_at(0, 0), 3.5);
    }
}
