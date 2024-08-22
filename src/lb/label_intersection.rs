use crate::TreeArena;
use rustc_hash::FxHashSet;

pub fn label_intersection_distance(t1: &TreeArena, t2: &TreeArena, max_distance: usize) -> usize {
    if t1.count().abs_diff(t2.count()) > max_distance {
        return t1.count().abs_diff(t2.count());
    }

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

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::*;

    #[test]
    fn test_label_intersection() {
        let t1 = crate::parsing::parse_tree(&CString::new("{a{b}{c}}").unwrap()).unwrap();
        let t2 = crate::parsing::parse_tree(&CString::new("{a{d}{c}}").unwrap()).unwrap();
        assert_eq!(
            label_intersection_distance(&t1, &t2, t1.count() + t2.count()),
            1
        )
    }
}
