use crate::{types::InvertedListLabelPostorderIndex, TreeArena};
use rustc_hash::FxHashSet;

pub fn label_intersection_distance(t1: &TreeArena, t2: &TreeArena) -> usize {
    let mut t1_label_set = FxHashSet::default();

    for n in t1.iter() {
        let l = n.get().as_str();
        t1_label_set.insert(l);
    }

    let mut intersection_size = 0;

    for n in t2.iter() {
        let l = n.get().as_str();
        if t1_label_set.contains(l) {
            intersection_size += 1;
        }
    }

    std::cmp::max(t1.count(), t2.count()) - intersection_size
}

pub fn bounded_label_intersection_distance(
    t1: &TreeArena,
    t2: &TreeArena,
    max_distance: usize,
) -> usize {
    if t1.count().abs_diff(t2.count()) > max_distance {
        return t1.count().abs_diff(t2.count());
    }
    let bigger_tree = std::cmp::max(t1.count(), t2.count());

    let mut t1_label_set = FxHashSet::default();

    for n in t1.iter() {
        let l = n.get().as_str();
        t1_label_set.insert(l);
    }

    let mut intersection_size = 0;

    let mut t2_cnt = t2.count();

    for n in t2.iter() {
        t2_cnt -= 1;
        let l = n.get().as_str();
        if t1_label_set.contains(l) {
            intersection_size += 1;
        }

        if bigger_tree - intersection_size - t2_cnt > max_distance {
            return max_distance + 1;
        }
    }

    bigger_tree - intersection_size
}

pub fn label_intersection_inverted_distance(
    t1: &InvertedListLabelPostorderIndex,
    t2: &InvertedListLabelPostorderIndex,
) -> i32 {
    use std::cmp::{max, min};
    let mut intersection_size = 0;
    for (label, node_cnt) in t1.inverted_list.iter() {
        if let Some(t2_node_cnt) = t2.inverted_list.get(label) {
            intersection_size += min(t2_node_cnt, node_cnt);
        }
    }

    max(t1.tree_size, t2.tree_size) as i32 - intersection_size
}

pub fn bounded_label_intersection_inverted_distance(
    t1: &InvertedListLabelPostorderIndex,
    t2: &InvertedListLabelPostorderIndex,
    max_distance: usize,
) -> i32 {
    use std::cmp::{max, min};
    let mut intersection_size = 0;
    let bigger_tree = max(t1.tree_size, t2.tree_size) as i32;

    // if all labels matched, but just the size difference was too much, just exit
    if t1.tree_size.abs_diff(t2.tree_size) > max_distance {
        return max_distance as i32 + 1;
    }

    for (label, node_cnt) in t1.inverted_list.iter() {
        let Some(t2_node_cnt) = t2.inverted_list.get(label) else {
            continue;
        };
        intersection_size += min(t2_node_cnt, node_cnt);

        if bigger_tree - intersection_size < max_distance as i32 {
            return bigger_tree - intersection_size;
        }
    }

    bigger_tree - intersection_size
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::*;

    #[test]
    fn test_label_intersection() {
        let t1 = crate::parsing::parse_tree(&CString::new("{a{b}{c}}").unwrap()).unwrap();
        let t2 = crate::parsing::parse_tree(&CString::new("{a{d}{c}}").unwrap()).unwrap();
        assert_eq!(label_intersection_distance(&t1, &t2), 1)
    }

    #[test]
    fn test_bounded_label_intersection_1() {
        let t1 = crate::parsing::parse_tree(&CString::new("{a{b}{c}{f}}").unwrap()).unwrap();
        let t2 = crate::parsing::parse_tree(&CString::new("{a{d}{e}}").unwrap()).unwrap();
        assert_eq!(bounded_label_intersection_distance(&t1, &t2, 1), 2)
    }

    #[test]
    fn test_bounded_label_intersection_2() {
        let t1 = crate::parsing::parse_tree(&CString::new("{a{b}{c}}").unwrap()).unwrap();
        let t2 = crate::parsing::parse_tree(&CString::new("{a{d}{e}}").unwrap()).unwrap();
        assert_eq!(bounded_label_intersection_distance(&t1, &t2, 3), 2)
    }

    #[test]
    fn test_bounded_label_intersection_3() {
        let t1 = crate::parsing::parse_tree(&CString::new("{a{b}{e}{f}}").unwrap()).unwrap();
        let t2 = crate::parsing::parse_tree(&CString::new("{a{d}{e}}").unwrap()).unwrap();
        assert_eq!(bounded_label_intersection_distance(&t1, &t2, 1), 2)
    }
}
