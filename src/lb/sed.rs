use pgrx::{prelude::*, PostgresType};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{parsing::parse_tree, types::tree_internals::id::NodeId, TreeArena};

/// Local label-interning dictionary used to translate string labels to i32 for
/// fast comparisons during SED/SED-STRUCT computation. Built fresh per call.
pub type LabelDict = FxHashMap<String, i32>;

#[inline]
fn intern(dict: &mut LabelDict, label: &str) -> i32 {
    if let Some(&id) = dict.get(label) {
        return id;
    }
    let id = dict.len() as i32;
    dict.insert(label.to_owned(), id);
    id
}

/// Trait abstracting a structural traversal character so the DP loop can be
/// generic over both `TraversalCharacter` (string label) and
/// `TraversalCharacterInt` (interned i32 label).
pub trait StructCell {
    type Label: PartialEq;
    fn label(&self) -> &Self::Label;
    fn sum(&self) -> i32;
    fn diff(&self) -> i32;
}

pub fn sed(t1: &SEDIndex, t2: &SEDIndex) -> usize {
    let (mut t1, mut t2) = (t1, t2);
    if t1.preorder.len() > t2.preorder.len() {
        (t1, t2) = (t2, t1);
    }

    let pre_dist = string_edit_distance(&t1.preorder, &t2.preorder);
    let post_dist = string_edit_distance(&t1.postorder, &t2.postorder);

    std::cmp::max(pre_dist, post_dist)
}

pub fn bounded_sed(t1: &SEDIndex, t2: &SEDIndex, k: usize) -> usize {
    if t1.tree_size.abs_diff(t2.tree_size) > k {
        return k + 1;
    }
    let (mut t1, mut t2) = (t1, t2);
    if t1.preorder.len() > t2.preorder.len() {
        (t1, t2) = (t2, t1);
    }
    let k = k + 1;
    let pre_dist = bounded_string_edit_distance(&t1.preorder, &t2.preorder, k);
    if pre_dist > k {
        return pre_dist;
    }
    let post_dist = bounded_string_edit_distance(&t1.postorder, &t2.postorder, k);

    std::cmp::max(pre_dist, post_dist)
}

#[derive(Debug, PostgresType, Serialize, Deserialize)]
#[inoutfuncs]
pub struct SEDIndex {
    pub preorder: Vec<String>,
    pub postorder: Vec<String>,
    pub tree_size: usize,
}

impl SEDIndex {
    pub fn index_tree(tree: &TreeArena) -> Self {
        let Some(root) = tree.iter().next() else {
            panic!("Unable to get root but tree is not empty!");
        };
        let root_id = tree.get_node_id(root).unwrap();

        let mut pre = Vec::with_capacity(tree.count());
        let mut post = Vec::with_capacity(tree.count());

        traverse(root_id, tree, &mut pre, &mut post);

        Self {
            tree_size: tree.count(),
            postorder: post,
            preorder: pre,
        }
    }
}

impl InOutFuncs for SEDIndex {
    fn input(input: &core::ffi::CStr) -> Self
    where
        Self: Sized,
    {
        Self::from(parse_tree(input).expect("failed to parse input tree"))
    }

    fn output(&self, buffer: &mut pgrx::StringInfo) {
        buffer.push_str(&self.postorder.join(":"));
        buffer.push_str(",");
        buffer.push_str(&self.preorder.join(":"));
    }
}

impl From<TreeArena> for SEDIndex {
    fn from(tree: TreeArena) -> Self {
        SEDIndex::index_tree(&tree)
    }
}

fn traverse(nid: NodeId, tree: &TreeArena, pre: &mut Vec<String>, post: &mut Vec<String>) {
    // i am here at the current root
    let label = tree.get(nid).unwrap().get();
    pre.push(label.clone());
    for cnid in nid.children(tree) {
        traverse(cnid, tree, pre, post);
    }
    post.push(label.clone());
}

/// Implements fastest known way to compute exact string edit between two strings
fn string_edit_distance(s1: &[String], s2: &[String]) -> usize {
    use std::cmp::min;
    // assumes size of s2 is smaller or equal than s1
    let s2len = s2.len();
    let mut cache: Vec<usize> = (1..s2len + 1).collect();
    let mut result = s2len;
    for (i, ca) in s1.iter().enumerate() {
        result = i + 1;
        let mut dist_b = i;

        for (j, cb) in s2.iter().enumerate() {
            let dist_a = dist_b + usize::from(ca != cb);
            unsafe {
                dist_b = *cache.get_unchecked(j);
                result = min(result + 1, min(dist_a, dist_b + 1));
                *cache.get_unchecked_mut(j) = result;
            }
        }
    }

    result
}

fn bounded_string_edit_distance(s1: &[String], s2: &[String], k: usize) -> usize {
    use std::cmp::{max, min};
    // assumes size of s2 is smaller or equal than s1
    let mut s1len = s1.len();
    let mut s2len = s2.len();
    // perform suffix trimming
    for _ in s1
        .iter()
        .rev()
        .zip(s2.iter().rev())
        .take_while(|(s1c, s2c)| s1c == s2c)
    {
        s1len -= 1;
        s2len -= 1;
        if s1len == 0 {
            break;
        }
    }

    let mut common_prefix = 0;

    // now prefix trimming
    for _ in s1.iter().zip(s2.iter()).take_while(|(s1c, s2c)| s1c == s2c) {
        common_prefix += 1;
        if common_prefix >= s1len {
            break;
        }
    }

    if s1len == 0 {
        return s2len;
    }

    // prefix trimming done
    let s1 = &s1[common_prefix..s1len];
    let s2 = &s2[common_prefix..s2len];

    s1len -= common_prefix;
    s2len -= common_prefix;
    // one string is gone by suffix and prefix trimming, so just return the remaining size
    if s1len == 0 {
        return s2len;
    }
    let s1len = s1len as i64;
    let s2len = s2len as i64;

    let threshold = min(s2len, k as i64);
    let size_diff = s2len - s1len;

    if threshold < size_diff {
        return threshold as usize;
    }

    let zero_k: i64 = ((if s1len < threshold { s1len } else { threshold }) >> 1) + 2;

    let arr_len = size_diff + (zero_k) * 2 + 2;

    let mut current_row = vec![-1i64; arr_len as usize];
    let mut next_row = vec![-1i64; arr_len as usize];
    let mut i = 0;
    let condition_row = size_diff + zero_k;
    let end_max = condition_row << 1;

    loop {
        i += 1;
        std::mem::swap(&mut next_row, &mut current_row);

        let start: i64;
        let mut next_cell: i64;
        let mut previous_cell: i64;
        let mut current_cell: i64 = -1;

        if i <= zero_k {
            start = -i + 1;
            next_cell = i - 2i64;
        } else {
            start = i - (zero_k << 1) + 1;
            unsafe {
                next_cell = *current_row.get_unchecked((zero_k + start) as usize);
            }
        }

        let end: i64;
        if i <= condition_row {
            end = i;
            unsafe {
                *next_row.get_unchecked_mut((zero_k + i) as usize) = -1;
            }
        } else {
            end = end_max - i;
        }

        let mut row_index = (start + zero_k) as usize;

        let mut t;

        for q in start..end {
            previous_cell = current_cell;
            current_cell = next_cell;
            unsafe {
                next_cell = *current_row.get_unchecked(row_index + 1);
            }

            // max()
            t = max(max(current_cell + 1, previous_cell), next_cell + 1);

            unsafe {
                while t < s1len
                    && (t + q) < s2len
                    && s1.get_unchecked(t as usize) == s2.get_unchecked((t + q) as usize)
                {
                    t += 1;
                }
            }

            unsafe {
                *next_row.get_unchecked_mut(row_index) = t;
            }
            row_index += 1;
        }

        unsafe {
            if !(*next_row.get_unchecked(condition_row as usize) < s1len && i <= threshold) {
                break (i - 1) as usize;
            }
        }
    }
}

// ============================================================================
// Optimized bounded SED — budget-constrained band intersection
// (Berghel-Roach with diagonal pruning based on remaining edit budget)
// ============================================================================

pub fn bounded_sed_opt(t1: &SEDIndex, t2: &SEDIndex, k: usize) -> usize {
    if t1.tree_size.abs_diff(t2.tree_size) > k {
        return k + 1;
    }
    let (mut t1, mut t2) = (t1, t2);
    if t1.preorder.len() > t2.preorder.len() {
        (t1, t2) = (t2, t1);
    }

    let pre_dist = bounded_string_edit_distance_opt(&t1.preorder, &t2.preorder, k);
    if pre_dist > k {
        return k + 1;
    }
    let post_dist = bounded_string_edit_distance_opt(&t1.postorder, &t2.postorder, k);
    if post_dist > k {
        return k + 1;
    }
    std::cmp::max(pre_dist, post_dist)
}

fn bounded_string_edit_distance_opt<T: PartialEq>(s1: &[T], s2: &[T], k: usize) -> usize {
    use std::cmp::{max, min};
    let (s1, s2) = if s1.len() <= s2.len() {
        (s1, s2)
    } else {
        (s2, s1)
    };
    let s1len = s1.len() as i64;
    let s2len = s2.len() as i64;

    let threshold = min(s2len, k as i64);
    let size_diff = s2len - s1len;
    if size_diff > threshold {
        return usize::MAX;
    }

    let zero_k: i64 = ((if s1len < threshold { s1len } else { threshold }) >> 1) + 2;
    let arr_len = size_diff + (zero_k) * 2 + 2;
    let condition_diag = size_diff + zero_k;
    let end_max = condition_diag << 1;

    let mut current_row = vec![-1i64; arr_len as usize];
    let mut next_row = vec![-1i64; arr_len as usize];

    for i in 1..=threshold + 1 {
        std::mem::swap(&mut next_row, &mut current_row);

        let original_start: i64 = if i <= zero_k {
            -i + 1
        } else {
            i - (zero_k << 1) + 1
        };

        let original_end: i64;
        if i <= condition_diag {
            original_end = i;
            unsafe {
                *next_row.get_unchecked_mut((zero_k + i) as usize) = -1;
            }
        } else {
            original_end = end_max - i;
        }

        // Budget-constrained band: only diagonals reachable within remaining edits
        let budget = threshold - (i - 1);
        let (min_valid_diag, max_valid_diag) = if budget <= 0 {
            (size_diff, size_diff)
        } else {
            (size_diff - budget, size_diff + budget)
        };

        let start = max(original_start, min_valid_diag);
        let end = min(original_end, max_valid_diag + 1);

        let mut current_cell: i64;
        let mut next_cell: i64;
        let mut previous_cell: i64;

        if i <= zero_k && start == original_start {
            current_cell = -1;
            next_cell = i - 2i64;
        } else {
            unsafe {
                let start_idx = (zero_k + start) as usize;
                current_cell = if start > original_start && start_idx > 0 {
                    *current_row.get_unchecked(start_idx - 1)
                } else {
                    -1
                };
                next_cell = *current_row.get_unchecked(start_idx);
            }
        }

        let mut row_index = (start + zero_k) as usize - 1;
        let mut t;

        for q in start..end {
            row_index += 1;
            previous_cell = current_cell;
            current_cell = next_cell;
            unsafe {
                next_cell = *current_row.get_unchecked(row_index + 1);
            }

            t = max(max(current_cell + 1, previous_cell), next_cell + 1);

            unsafe {
                while t < s1len
                    && (t + q) < s2len
                    && s1.get_unchecked(t as usize) == s2.get_unchecked((t + q) as usize)
                {
                    t += 1;
                }
            }

            unsafe {
                *next_row.get_unchecked_mut(row_index) = t;
            }
        }

        unsafe {
            let condition_value = *next_row.get_unchecked(condition_diag as usize);
            if condition_value >= s1len {
                return (i - 1) as usize;
            }
        }
    }

    usize::MAX
}

// ============================================================================
// SED-STRUCT — tighter lower bound using tree structural info (sum, diff)
// Each traversal character carries the node's positional features so the DP
// can prune extensions that violate the structural budget.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalCharacter {
    pub label: String,
    pub sum: i32,
    pub diff: i32,
}

impl StructCell for TraversalCharacter {
    type Label = String;
    #[inline] fn label(&self) -> &String { &self.label }
    #[inline] fn sum(&self) -> i32 { self.sum }
    #[inline] fn diff(&self) -> i32 { self.diff }
}

#[derive(Debug, PostgresType, Serialize, Deserialize)]
#[inoutfuncs]
pub struct SEDStructIndex {
    pub first_traversal: Vec<TraversalCharacter>,  // preorder with (following+descendant, descendant-following)
    pub second_traversal: Vec<TraversalCharacter>, // reversed-postorder with (preceding+ancestor, ancestor-preceding)
    pub tree_size: usize,
}

impl SEDStructIndex {
    pub fn index_tree(tree: &TreeArena) -> Self {
        let Some(root) = tree.iter().next() else {
            panic!("Unable to get root but tree is not empty!");
        };
        let root_id = tree.get_node_id(root).unwrap();

        let tree_size = tree.count();
        let mut preorder = Vec::with_capacity(tree_size);
        let mut reversed_postorder = Vec::with_capacity(tree_size);

        let mut postorder_id = 0usize;
        let mut depth = 0usize;
        traverse_with_info(
            root_id,
            tree,
            tree_size,
            &mut preorder,
            &mut reversed_postorder,
            &mut postorder_id,
            &mut depth,
        );

        reversed_postorder.reverse();

        Self {
            first_traversal: preorder,
            second_traversal: reversed_postorder,
            tree_size,
        }
    }
}

impl From<TreeArena> for SEDStructIndex {
    fn from(tree: TreeArena) -> Self {
        SEDStructIndex::index_tree(&tree)
    }
}

impl InOutFuncs for SEDStructIndex {
    fn input(input: &core::ffi::CStr) -> Self
    where
        Self: Sized,
    {
        Self::from(parse_tree(input).expect("failed to parse input tree"))
    }

    fn output(&self, buffer: &mut pgrx::StringInfo) {
        for (i, tc) in self.first_traversal.iter().enumerate() {
            if i > 0 {
                buffer.push_str("|");
            }
            buffer.push_str(&format!("{}:{}:{}", tc.label, tc.sum, tc.diff));
        }
        buffer.push_str(",");
        for (i, tc) in self.second_traversal.iter().enumerate() {
            if i > 0 {
                buffer.push_str("|");
            }
            buffer.push_str(&format!("{}:{}:{}", tc.label, tc.sum, tc.diff));
        }
    }
}

fn traverse_with_info(
    nid: NodeId,
    tree: &TreeArena,
    tree_size: usize,
    preorder: &mut Vec<TraversalCharacter>,
    reversed_postorder: &mut Vec<TraversalCharacter>,
    postorder_id: &mut usize,
    depth: &mut usize,
) -> usize {
    let mut subtree_size = 1;
    *depth += 1;

    let label = tree.get(nid).unwrap().get();

    let pre_idx = preorder.len();
    preorder.push(TraversalCharacter {
        label: label.clone(),
        sum: 0,
        diff: 0,
    });
    reversed_postorder.push(TraversalCharacter {
        label: label.clone(),
        sum: 0,
        diff: 0,
    });

    for cnid in nid.children(tree) {
        subtree_size += traverse_with_info(
            cnid,
            tree,
            tree_size,
            preorder,
            reversed_postorder,
            postorder_id,
            depth,
        );
    }

    *depth -= 1;
    *postorder_id += 1;

    let preceding = (*postorder_id).saturating_sub(subtree_size);
    let following = tree_size.saturating_sub(*postorder_id + *depth);
    let descendant = subtree_size as i32 - 1;
    let ancestor = *depth as i32;

    let pre = preorder.get_mut(pre_idx).unwrap();
    pre.sum = following as i32 + descendant;
    pre.diff = descendant - following as i32;

    let rev_post = reversed_postorder.get_mut(pre_idx).unwrap();
    rev_post.sum = preceding as i32 + ancestor;
    rev_post.diff = ancestor - preceding as i32;

    subtree_size
}

pub fn bounded_sed_struct(t1: &SEDStructIndex, t2: &SEDStructIndex, k: usize) -> usize {
    if t1.tree_size.abs_diff(t2.tree_size) > k {
        return k + 1;
    }
    let (mut t1, mut t2) = (t1, t2);
    if t1.first_traversal.len() > t2.first_traversal.len() {
        (t1, t2) = (t2, t1);
    }

    let first_dist =
        bounded_string_edit_distance_with_structure(&t1.first_traversal, &t2.first_traversal, k);
    if first_dist > k {
        return k + 1;
    }
    let second_dist =
        bounded_string_edit_distance_with_structure(&t1.second_traversal, &t2.second_traversal, k);
    if second_dist > k {
        return k + 1;
    }
    std::cmp::max(first_dist, second_dist)
}

pub fn bounded_string_edit_distance_with_structure<T: StructCell>(
    s1: &[T],
    s2: &[T],
    k: usize,
) -> usize {
    use std::cmp::{max, min};
    let s1len = s1.len() as i32;
    let s2len = s2.len() as i32;
    let size_diff = s2len - s1len;
    let threshold = min(s2len, k as i32);

    let zero_k: i32 = threshold + 1;
    let array_size = (2 * threshold + 3) as usize;

    let mut current_row = vec![(-1i32, true); array_size];
    let mut next_row = vec![(-1i32, true); array_size];
    let target_diagonal = size_diff + zero_k;
    let target_diagonal_idx = target_diagonal as usize;
    let end_max = target_diagonal << 1;

    for i in 1..=threshold + 1 {
        std::mem::swap(&mut next_row, &mut current_row);

        let original_start: i32 = if i <= zero_k {
            -i + 1
        } else {
            i - (zero_k << 1) + 1
        };

        let original_end: i32;
        if i <= target_diagonal {
            original_end = i;
            unsafe {
                *next_row.get_unchecked_mut((zero_k + i) as usize) = (-1, true);
            }
        } else {
            original_end = end_max - i;
        }

        let budget = k as i32 - (i - 1);
        let (min_valid_diag, max_valid_diag) = if budget <= 0 {
            (size_diff, size_diff)
        } else {
            (size_diff - budget, size_diff + budget)
        };

        let start = max(original_start, min_valid_diag);
        let end = min(original_end, max_valid_diag + 1);

        let mut current_cell: i32;
        let mut next_cell: i32;
        let mut previous_cell: i32;
        let mut next_allowed_substitution: bool;

        if i <= zero_k && start == original_start {
            current_cell = -1;
            next_cell = i - 2i32;
            next_allowed_substitution = true;
        } else {
            unsafe {
                let start_idx = (zero_k + start) as usize;
                current_cell = if start > original_start && start_idx > 0 {
                    current_row.get_unchecked(start_idx - 1).0
                } else {
                    -1
                };
                (next_cell, next_allowed_substitution) = *current_row.get_unchecked(start_idx);
            }
        }

        let mut diagonal_index: usize = (start + zero_k).try_into().unwrap();

        let mut max_row_number;
        let allowed_edits = i - 1;
        let mut can_substitute: bool;

        for diag_offset in start..end {
            previous_cell = current_cell;
            current_cell = next_cell;
            can_substitute = next_allowed_substitution;
            unsafe {
                (next_cell, next_allowed_substitution) =
                    *current_row.get_unchecked(diagonal_index + 1);
            }

            unsafe {
                max_row_number = max(
                    current_cell + (if can_substitute { 1 } else { 0 }),
                    max(previous_cell, next_cell + 1),
                );

                if !can_substitute && max_row_number == current_cell {
                    *next_row.get_unchecked_mut(diagonal_index) = (max_row_number, false);
                    diagonal_index += 1;
                    continue;
                }
            }

            unsafe {
                let k_i32 = k as i32;
                let mut struct_ok = false;

                while max_row_number < s1len && (max_row_number + diag_offset) < s2len {
                    let c1 = s1.get_unchecked(max_row_number as usize);
                    let c2 = s2.get_unchecked((max_row_number + diag_offset) as usize);

                    let char_eq = c1.label() == c2.label();
                    struct_ok = (allowed_edits + (c1.sum() - c2.sum()).abs() <= k_i32)
                        && (allowed_edits + (c1.diff() - c2.diff()).abs() <= k_i32);

                    if !char_eq || !struct_ok {
                        break;
                    }
                    max_row_number += 1;
                }

                *next_row.get_unchecked_mut(diagonal_index) = (max_row_number, struct_ok);
            }

            diagonal_index += 1;
        }

        unsafe {
            if next_row.get_unchecked(target_diagonal_idx).0 >= s1len {
                return (i - 1) as usize;
            }
        }
    }

    usize::MAX
}

// ============================================================================
// Integer-based variants — labels interned via a local LabelDict for fast
// PartialEq during the inner DP loop. Built per-call from two TreeArenas.
// These types are NOT exposed as Postgres types because the interning is only
// meaningful within a single comparison.
// ============================================================================

#[derive(Debug)]
pub struct SEDIndexInt {
    pub preorder: Vec<i32>,
    pub postorder: Vec<i32>,
    pub tree_size: usize,
}

#[derive(Debug, Clone)]
pub struct TraversalCharacterInt {
    pub label: i32,
    pub sum: i32,
    pub diff: i32,
}

impl StructCell for TraversalCharacterInt {
    type Label = i32;
    #[inline] fn label(&self) -> &i32 { &self.label }
    #[inline] fn sum(&self) -> i32 { self.sum }
    #[inline] fn diff(&self) -> i32 { self.diff }
}

#[derive(Debug)]
pub struct SEDStructIndexInt {
    pub first_traversal: Vec<TraversalCharacterInt>,  // preorder
    pub second_traversal: Vec<TraversalCharacterInt>, // reversed-postorder
    pub tree_size: usize,
}

/// Build two integer-interned SEDIndexInt's sharing the same local dictionary.
pub fn build_sed_indices_int(t1: &TreeArena, t2: &TreeArena) -> (SEDIndexInt, SEDIndexInt) {
    let mut dict: LabelDict = FxHashMap::default();
    let i1 = sed_index_int(t1, &mut dict);
    let i2 = sed_index_int(t2, &mut dict);
    (i1, i2)
}

/// Build two integer-interned SEDStructIndexInt's sharing the same local dictionary.
pub fn build_sed_struct_indices_int(
    t1: &TreeArena,
    t2: &TreeArena,
) -> (SEDStructIndexInt, SEDStructIndexInt) {
    let mut dict: LabelDict = FxHashMap::default();
    let i1 = sed_struct_index_int(t1, &mut dict);
    let i2 = sed_struct_index_int(t2, &mut dict);
    (i1, i2)
}

fn sed_index_int(tree: &TreeArena, dict: &mut LabelDict) -> SEDIndexInt {
    let Some(root) = tree.iter().next() else {
        panic!("Unable to get root but tree is not empty!");
    };
    let root_id = tree.get_node_id(root).unwrap();

    let mut pre = Vec::with_capacity(tree.count());
    let mut post = Vec::with_capacity(tree.count());

    traverse_int(root_id, tree, dict, &mut pre, &mut post);

    SEDIndexInt {
        tree_size: tree.count(),
        preorder: pre,
        postorder: post,
    }
}

fn traverse_int(
    nid: NodeId,
    tree: &TreeArena,
    dict: &mut LabelDict,
    pre: &mut Vec<i32>,
    post: &mut Vec<i32>,
) {
    let label = tree.get(nid).unwrap().get();
    let id = intern(dict, label);
    pre.push(id);
    for cnid in nid.children(tree) {
        traverse_int(cnid, tree, dict, pre, post);
    }
    post.push(id);
}

fn sed_struct_index_int(tree: &TreeArena, dict: &mut LabelDict) -> SEDStructIndexInt {
    let Some(root) = tree.iter().next() else {
        panic!("Unable to get root but tree is not empty!");
    };
    let root_id = tree.get_node_id(root).unwrap();

    let tree_size = tree.count();
    let mut preorder = Vec::with_capacity(tree_size);
    let mut reversed_postorder = Vec::with_capacity(tree_size);

    let mut postorder_id = 0usize;
    let mut depth = 0usize;
    traverse_with_info_int(
        root_id,
        tree,
        tree_size,
        dict,
        &mut preorder,
        &mut reversed_postorder,
        &mut postorder_id,
        &mut depth,
    );

    reversed_postorder.reverse();

    SEDStructIndexInt {
        first_traversal: preorder,
        second_traversal: reversed_postorder,
        tree_size,
    }
}

fn traverse_with_info_int(
    nid: NodeId,
    tree: &TreeArena,
    tree_size: usize,
    dict: &mut LabelDict,
    preorder: &mut Vec<TraversalCharacterInt>,
    reversed_postorder: &mut Vec<TraversalCharacterInt>,
    postorder_id: &mut usize,
    depth: &mut usize,
) -> usize {
    let mut subtree_size = 1;
    *depth += 1;

    let label = tree.get(nid).unwrap().get();
    let id = intern(dict, label);

    let pre_idx = preorder.len();
    preorder.push(TraversalCharacterInt { label: id, sum: 0, diff: 0 });
    reversed_postorder.push(TraversalCharacterInt { label: id, sum: 0, diff: 0 });

    for cnid in nid.children(tree) {
        subtree_size += traverse_with_info_int(
            cnid,
            tree,
            tree_size,
            dict,
            preorder,
            reversed_postorder,
            postorder_id,
            depth,
        );
    }

    *depth -= 1;
    *postorder_id += 1;

    let preceding = (*postorder_id).saturating_sub(subtree_size);
    let following = tree_size.saturating_sub(*postorder_id + *depth);
    let descendant = subtree_size as i32 - 1;
    let ancestor = *depth as i32;

    let pre = preorder.get_mut(pre_idx).unwrap();
    pre.sum = following as i32 + descendant;
    pre.diff = descendant - following as i32;

    let rev_post = reversed_postorder.get_mut(pre_idx).unwrap();
    rev_post.sum = preceding as i32 + ancestor;
    rev_post.diff = ancestor - preceding as i32;

    subtree_size
}

pub fn bounded_sed_opt_int(t1: &SEDIndexInt, t2: &SEDIndexInt, k: usize) -> usize {
    if t1.tree_size.abs_diff(t2.tree_size) > k {
        return k + 1;
    }
    let (mut t1, mut t2) = (t1, t2);
    if t1.preorder.len() > t2.preorder.len() {
        (t1, t2) = (t2, t1);
    }

    let pre_dist = bounded_string_edit_distance_opt(&t1.preorder, &t2.preorder, k);
    if pre_dist > k {
        return k + 1;
    }
    let post_dist = bounded_string_edit_distance_opt(&t1.postorder, &t2.postorder, k);
    if post_dist > k {
        return k + 1;
    }
    std::cmp::max(pre_dist, post_dist)
}

pub fn bounded_sed_struct_int(
    t1: &SEDStructIndexInt,
    t2: &SEDStructIndexInt,
    k: usize,
) -> usize {
    if t1.tree_size.abs_diff(t2.tree_size) > k {
        return k + 1;
    }
    let (mut t1, mut t2) = (t1, t2);
    if t1.first_traversal.len() > t2.first_traversal.len() {
        (t1, t2) = (t2, t1);
    }

    let first_dist =
        bounded_string_edit_distance_with_structure(&t1.first_traversal, &t2.first_traversal, k);
    if first_dist > k {
        return k + 1;
    }
    let second_dist =
        bounded_string_edit_distance_with_structure(&t1.second_traversal, &t2.second_traversal, k);
    if second_dist > k {
        return k + 1;
    }
    std::cmp::max(first_dist, second_dist)
}
