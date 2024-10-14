use crate::types::tree_structural::{
    RegionNumType, SplitStructuralFilterTuple, SplitStructuralVec, StructuralFilter, StructuralVec,
};

#[inline(always)]
fn split_svec_l1(n1: &SplitStructuralVec, n2: &SplitStructuralVec) -> i32 {
    use std::cmp::{max, min};
    // for each axis, take the maximum of L1 difference
    let sum = n1
        .svec
        .mapping_regions
        .iter()
        .zip(&n2.svec.mapping_regions)
        .enumerate()
        .fold(0, |acc, (region, (a, b))| {
            acc + (max(a, b)
                - n1.mapping_region_splits[region]
                    .iter()
                    .zip(&n2.mapping_region_splits[region])
                    .map(|(s1, s2)| min(s1, s2))
                    .sum::<RegionNumType>())
        });
    sum as i32
}

#[inline(always)]
fn svec_l1(n1: &StructuralVec, n2: &StructuralVec) -> i32 {
    n1.mapping_regions
        .iter()
        .zip(n2.mapping_regions.iter())
        .fold(0, |acc, (a, b)| acc + a.abs_diff(*b)) as i32
}

/// Given two sets
pub fn ted(s1: &StructuralFilter, s2: &StructuralFilter, k: i32) -> i32 {
    use std::cmp::max;
    let bigger = max(s1.0, s2.0);

    if s1.0.abs_diff(s2.0) > k as usize {
        return k + 1;
    }

    let overlap = get_nodes_overlap_with_region_distance(s1, s2, k as usize, svec_l1);

    (bigger - overlap) as i32
}

pub fn ted_variant(
    s1: &SplitStructuralFilterTuple,
    s2: &SplitStructuralFilterTuple,
    k: usize,
) -> usize {
    if s1.0.abs_diff(s2.0) > k {
        return k + 1;
    }
    let bigger = std::cmp::max(s1.0, s2.0);
    let mut overlap = 0;
    let k = k as i32;
    for (lblid, set1) in s1.1.iter() {
        if let Some(set2) = s2.1.get(lblid) {
            if set1.base.weight == 1 && set2.base.weight == 1 {
                let l1_region_distance = split_svec_l1(&set1.struct_vec[0], &set2.struct_vec[0]);
                if l1_region_distance <= k {
                    overlap += 1;
                }
                continue;
            }

            let (s1c, s2c) = if set2.base.weight < set1.base.weight {
                (set2, set1)
            } else {
                (set1, set2)
            };

            for n1 in s1c.struct_vec.iter() {
                let mut k_window = n1.svec.postorder_id.saturating_sub(k);
                k_window = std::cmp::min(k_window, 0);

                // apply postorder filter
                let s2clen = s2c.struct_vec.len() as i32;
                for n2 in s2c.struct_vec.iter() {
                    if k_window < s2clen && n2.svec.postorder_id < k_window {
                        continue;
                    }

                    if n2.svec.postorder_id > k + n1.svec.postorder_id {
                        break;
                    }
                    let l1_region_distance = split_svec_l1(n1, n2);

                    if l1_region_distance <= k {
                        overlap += 1;
                        break;
                    }
                }
            }
        }
    }

    bigger.saturating_sub(overlap)
}

fn get_nodes_overlap_with_region_distance(
    s1: &StructuralFilter,
    s2: &StructuralFilter,
    k: usize,
    region_distance_closure: impl Fn(&StructuralVec, &StructuralVec) -> i32,
) -> usize {
    let mut overlap = 0;
    let k = k as i32;
    for (lblid, set1) in s1.1.iter() {
        if let Some(set2) = s2.1.get(lblid) {
            if set1.base.weight == 1 && set2.base.weight == 1 {
                let l1_region_distance =
                    region_distance_closure(&set1.struct_vec[0], &set2.struct_vec[0]);
                if l1_region_distance <= k {
                    overlap += 1;
                }
                continue;
            }

            let (s1c, s2c) = if set2.base.weight < set1.base.weight {
                (set2, set1)
            } else {
                (set1, set2)
            };

            for n1 in s1c.struct_vec.iter() {
                let mut k_window = n1.postorder_id.saturating_sub(k);
                k_window = std::cmp::min(k_window, 0);
                // apply postorder filter
                let s2clen = s2c.struct_vec.len() as i32;
                for n2 in s2c.struct_vec.iter() {
                    if k_window < s2clen && n2.postorder_id < k_window {
                        continue;
                    }

                    if n2.postorder_id > k + n1.postorder_id {
                        break;
                    }
                    let l1_region_distance = region_distance_closure(n1, n2);

                    if l1_region_distance <= k {
                        overlap += 1;
                        break;
                    }
                }
            }
        }
    }

    overlap
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::*;
    use crate::parsing::parse_tree;
    use crate::types::tree_structural::{
        LabelSetConverter, LabelSetElement, LabelSetElementBase, SplitStructuralVec,
        StructuralFilter, StructuralVec,
    };

    #[test]
    fn test_set_converting() {
        let t1input = CString::new("{a{b}{a{b}{c}{a}}{b}}").unwrap();
        let t2input = CString::new("{a{c}{b{a{a}{b}{c}}}}").unwrap();
        let t1 = parse_tree(&t1input).unwrap();
        let t2 = parse_tree(&t2input).unwrap();
        let v = vec![t1, t2];
        let mut sc = LabelSetConverter::default();
        let sets = sc.create(&v);
        let lse_for_a = LabelSetElement {
            base: LabelSetElementBase {
                id: "a".to_string(),
                weight: 3,
                weigh_so_far: 0,
            },
            struct_vec: vec![
                StructuralVec {
                    label_id: "a".to_string(),
                    mapping_regions: [3, 2, 1, 0],
                    postorder_id: 4,
                },
                StructuralVec {
                    label_id: "a".to_string(),
                    mapping_regions: [1, 1, 1, 3],
                    postorder_id: 5,
                },
                StructuralVec {
                    label_id: "a".to_string(),
                    mapping_regions: [0, 0, 0, 6],
                    postorder_id: 7,
                },
            ],
        };

        let lse_for_b = LabelSetElement {
            base: LabelSetElementBase {
                id: "b".to_string(),
                weight: 3,
                weigh_so_far: 0,
            },
            struct_vec: vec![
                StructuralVec {
                    label_id: "b".to_string(),
                    mapping_regions: [0, 1, 5, 0],
                    postorder_id: 1,
                },
                StructuralVec {
                    label_id: "b".to_string(),
                    mapping_regions: [1, 2, 3, 0],
                    postorder_id: 2,
                },
                StructuralVec {
                    label_id: "b".to_string(),
                    mapping_regions: [5, 1, 0, 0],
                    postorder_id: 6,
                },
            ],
        };

        assert_eq!(sets[0].1.get("a").unwrap(), &lse_for_a);
        assert_eq!(sets[0].1.get("b").unwrap(), &lse_for_b);
    }

    #[test]
    fn test_struct_ted() {
        let t1input = CString::new("{a{b}{a{b}{c}{a}}{b}}").unwrap();
        let t2input = CString::new("{a{c}{b{a{a}{b}{c}}}}").unwrap();
        let t1 = parse_tree(&t1input).unwrap();
        let t2 = parse_tree(&t2input).unwrap();
        let v = vec![t1, t2];
        let mut sc = LabelSetConverter::default();
        let sets = sc.create(&v);

        let lb = ted(&sets[0], &sets[1], 4);

        assert_eq!(lb, 2);
    }
}
