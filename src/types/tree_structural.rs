use pgrx::{InOutFuncs, PostgresType};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::parsing::{parse_tree, LabelId, ParsedTree};

use super::tree_internals::id::NodeId;

pub type StructHashMap = FxHashMap<LabelId, LabelSetElement>;
pub type SplitStructHashMap = FxHashMap<LabelId, SplitLabelSetElement>;
pub type LabelDict = FxHashMap<String, usize>;

pub type RegionNumType = i32;

pub const REGION_LEFT_IDX: usize = 0;
/// ancestors
pub const REGION_ANC_IDX: usize = 1;

pub const REGION_RIGHT_IDX: usize = 2;
/// descendants
pub const REGION_DESC_IDX: usize = 3;

/// The building block for structural filter, holds information about
/// the count of ancestral nodes, descendants nodes, to the left and to the right
// difference between children and descendants? Children nodes are only 1 level below current node level
// while descendants are all nodes below the current node
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralVec {
    pub label_id: LabelId,
    /// Id of postorder tree traversal
    pub postorder_id: usize,
    /// Vector of number of nodes to the left, ancestors, nodes to right and descendants
    pub mapping_regions: [RegionNumType; 4],
}

/// This is an element holding relevant data of a set.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelSetElementBase {
    pub id: LabelId,
    pub weight: usize,
    pub weigh_so_far: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelSetElement {
    pub base: LabelSetElementBase,
    pub struct_vec: Vec<StructuralVec>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SplitLabelSetElement {
    pub base: LabelSetElementBase,
    pub struct_vec: Vec<SplitStructuralVec>,
}

/// Base struct tuple for structural filter
#[derive(Clone, Debug, PostgresType, Serialize, Deserialize, PartialEq)]
#[inoutfuncs]
pub struct StructuralFilterTuple(pub usize, pub StructHashMap);

impl InOutFuncs for StructuralFilterTuple {
    fn input(input: &core::ffi::CStr) -> Self
    where
        Self: Sized,
    {
        let tree = parse_tree(input).expect("failed to parse input tree");
        let mut lsc = LabelSetConverter::default();
        let mut tmp = lsc.create(&vec![tree]);
        let Some(sft) = tmp.pop() else {
            panic!("Unable to convert tree into structural tuple")
        };
        sft
    }

    fn output(&self, buffer: &mut pgrx::StringInfo) {
        let output = serde_json::to_string(self).expect("Failed to serialize");
        buffer.push_str(&output);
    }
}

#[derive(Clone, Debug)]
pub struct SplitStructuralFilterTuple(pub usize, pub SplitStructHashMap);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SplitStructuralVec {
    pub svec: StructuralVec,
    pub mapping_region_splits: [[RegionNumType; LabelSetConverter::MAX_SPLIT]; 4],
}

/// Takes a collection of trees and converts them into a collection of label
/// sets. A label set consists of labels and each label holds all nodes with that
/// label. The labels are substituted with their inverted label frequency number.
/// The labels in the sets are sorted by the global inverted frequency ordering
/// of the input collection.
#[derive(Debug, Default)]
pub struct LabelSetConverter {
    actual_depth: [RegionNumType; Self::MAX_SPLIT],
    actual_pre_order_number: [RegionNumType; Self::MAX_SPLIT],
    tree_size_by_split_id: [RegionNumType; Self::MAX_SPLIT],
}

impl LabelSetConverter {
    pub const MAX_SPLIT: usize = 4;

    pub fn create_split<F>(
        &mut self,
        trees: &[ParsedTree],
        mut split: F,
    ) -> Vec<SplitStructuralFilterTuple>
    where
        F: FnMut(&LabelId) -> usize,
    {
        // add one because range are end exclusive
        // frequency vector of pair (label weight, labelId)
        let mut sets_collection = Vec::with_capacity(trees.len());
        for tree in trees.iter() {
            // contains structural vectors for the current tree
            // is it a hash map of Label -> Vec<StructVec>
            let mut record_labels = SplitStructHashMap::default();

            let Some(root) = tree.iter().next() else {
                panic!("tree is empty");
            };
            let root_id = tree.get_node_id(root).unwrap();
            // for recursive postorder traversal
            let mut postorder_id = 0;

            for n in root_id.descendants(tree) {
                let root_label = tree.get(n).unwrap().get();
                let split_id = split(root_label);
                self.tree_size_by_split_id[split_id] += 1;
            }

            // array of records stored in sets_collection
            self.create_split_record(
                &root_id,
                tree,
                &mut postorder_id,
                &mut record_labels,
                &mut split,
            );

            // reset state variables needed for positional evaluation
            self.actual_depth = [0; Self::MAX_SPLIT];
            self.actual_pre_order_number = [0; Self::MAX_SPLIT];
            self.tree_size_by_split_id = [0; Self::MAX_SPLIT];
            sets_collection.push(SplitStructuralFilterTuple(tree.count(), record_labels));
        }
        sets_collection
    }

    pub fn create(&mut self, trees: &[ParsedTree]) -> Vec<StructuralFilterTuple> {
        // add one because range are end exclusive
        // frequency vector of pair (label weight, labelId)
        let mut sets_collection = Vec::with_capacity(trees.len());
        for tree in trees.iter() {
            // contains structural vectors for the current tree
            // is it a hash map of Label -> Vec<StructVec>
            let mut record_labels = StructHashMap::default();

            let Some(root) = tree.iter().next() else {
                panic!("tree is empty");
            };
            let root_id = tree.get_node_id(root).unwrap();
            // for recursive postorder traversal
            let mut postorder_id = 0;

            self.tree_size_by_split_id[0] = tree.count() as RegionNumType;

            // array of records stored in sets_collection
            self.create_record(&root_id, tree, &mut postorder_id, &mut record_labels);

            // reset state variables needed for positional evaluation
            self.actual_depth[0] = 0;
            self.actual_pre_order_number[0] = 0;
            self.tree_size_by_split_id[0] = 0;
            sets_collection.push(StructuralFilterTuple(tree.count(), record_labels));
        }
        sets_collection
    }

    /*
    pub fn create_with_frequency(
        &mut self,
        trees: &[ParsedTree],
        label_dict: &LabelDict,
        split: &impl Fn(&LabelId) -> usize,
    ) -> Vec<StructuralFilterTuple> {
        // add one because range are end exclusive
        let max_label_id = label_dict.values().max().unwrap() + 1;
        // frequency vector of pair (label weight, labelId)
        let mut label_freq_count = (0..max_label_id as usize).map(|lid| (0, lid)).collect_vec();
        let mut sets_collection = self.create(trees, split);

        for record in sets_collection.iter() {
            for label_set_element in record.1.iter() {
                label_freq_count[label_set_element.1.id as usize].0 += label_set_element.1.weight;
            }
        }
        // sort the vector based on label frequency
        label_freq_count.sort_by(|a, b| a.0.cmp(&b.0));

        // label map list [labelId] = frequencyId
        let mut label_map_list = Vec::with_capacity(label_freq_count.len());
        for (i, (_, lbl_cnt_)) in label_freq_count.iter().enumerate() {
            label_map_list[*lbl_cnt_] = i as LabelId;
        }

        for record in sets_collection.iter_mut() {
            for i in 0..record.1.len() {
                record[i].1.id = label_map_list[record[i].1.id as usize];
            }

            record.1.sort_by(|a, b| a.id.cmp(&b.id));

            let mut weight_sum = 0;
            for se in record.1.iter_mut() {
                weight_sum += se.1.weight;
                se.1.weigh_so_far = weight_sum;
            }
        }

        sets_collection
    }
    */

    fn create_split_record<F>(
        &mut self,
        root_id: &NodeId,
        tree: &ParsedTree,
        postorder_id: &mut usize,
        record_labels: &mut SplitStructHashMap,
        split: &mut F,
    ) -> [RegionNumType; Self::MAX_SPLIT]
    where
        F: FnMut(&LabelId) -> usize,
    {
        // number of children = subtree_size - 1
        // subtree_size = 1 -> actual node + sum of children
        let mut subtree_size = [0; Self::MAX_SPLIT];
        let root_label = tree.get(*root_id).unwrap().get();
        let split_id = split(root_label);
        subtree_size[split_id] = 1;

        self.actual_depth[split_id] += 1;

        for cid in root_id.children(tree) {
            let sizes = self.create_split_record(&cid, tree, postorder_id, record_labels, split);

            for (zref, b) in subtree_size.iter_mut().zip(&sizes) {
                *zref += *b;
            }
        }

        *postorder_id += 1;
        self.actual_depth[split_id] -= 1;
        self.actual_pre_order_number[split_id] += 1;

        let mut mapping_splits = [[0; Self::MAX_SPLIT]; 4];
        for i in 0..Self::MAX_SPLIT {
            // let follow = self.tree_size_by_split_id[i]
            //     - (self.actual_pre_order_number[i] + self.actual_depth[i]);
            mapping_splits[REGION_LEFT_IDX][i] = self.actual_pre_order_number[i] - subtree_size[i];
            mapping_splits[REGION_ANC_IDX][i] = self.actual_depth[i];

            mapping_splits[REGION_RIGHT_IDX][i] = self.tree_size_by_split_id[i]
                - (self.actual_pre_order_number[i] + self.actual_depth[i]);

            mapping_splits[REGION_DESC_IDX][i] = subtree_size[i];
        }
        mapping_splits[REGION_DESC_IDX][split_id] -= 1;

        let mut mapping_regions = [0; 4];
        mapping_regions
            .iter_mut()
            .enumerate()
            .for_each(|(region_idx, region)| {
                *region = mapping_splits[region_idx].iter().sum();
            });

        let node_struct_vec = SplitStructuralVec {
            svec: StructuralVec {
                label_id: root_label.clone(),
                postorder_id: *postorder_id,
                mapping_regions,
                ..Default::default()
            },
            mapping_region_splits: mapping_splits,
        };

        if let Some(se) = record_labels.get_mut(root_label) {
            se.base.weight += 1;
            se.struct_vec.push(node_struct_vec);
        } else {
            let mut se = SplitLabelSetElement {
                base: LabelSetElementBase {
                    id: tree.get(*root_id).unwrap().get().clone(),
                    weight: 1,
                    ..LabelSetElementBase::default()
                },
                ..Default::default()
            };
            se.struct_vec.push(node_struct_vec);
            record_labels.insert(root_label.clone(), se);
        }
        subtree_size
    }

    fn create_record(
        &mut self,
        root_id: &NodeId,
        tree: &ParsedTree,
        postorder_id: &mut usize,
        record_labels: &mut StructHashMap,
    ) -> RegionNumType {
        // number of children = subtree_size - 1
        // subtree_size = 1 -> actual node + sum of children
        let mut subtree_size = 1;

        self.actual_depth[0] += 1;

        for cid in root_id.children(tree) {
            subtree_size += self.create_record(&cid, tree, postorder_id, record_labels);
        }

        *postorder_id += 1;
        self.actual_depth[0] -= 1;
        self.actual_pre_order_number[0] += 1;

        let root_label = tree.get(*root_id).unwrap().get();
        let node_struct_vec = StructuralVec {
            postorder_id: *postorder_id,
            label_id: root_label.clone(),
            mapping_regions: [
                (self.actual_pre_order_number[0] - subtree_size),
                self.actual_depth[0],
                (self.tree_size_by_split_id[0]
                    - (self.actual_pre_order_number[0] + self.actual_depth[0])),
                (subtree_size - 1),
            ],
        };

        if let Some(se) = record_labels.get_mut(root_label) {
            se.base.weight += 1;
            se.struct_vec.push(node_struct_vec);
        } else {
            let mut se = LabelSetElement {
                base: LabelSetElementBase {
                    id: tree.get(*root_id).unwrap().get().clone(),
                    weight: 1,
                    ..LabelSetElementBase::default()
                },
                ..LabelSetElement::default()
            };
            se.struct_vec.push(node_struct_vec);
            record_labels.insert(root_label.clone(), se);
        }
        subtree_size
    }
}
