use std::collections::HashMap;

use pgrx::{InOutFuncs, PostgresType};
use serde::{Deserialize, Serialize};

use super::{tree_internals::id::NodeId, TreeArena};
use crate::parsing::{parse_tree, LabelId, ParsedTree};
type InvListLblPost = HashMap<LabelId, i32>;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, PostgresType)]
#[inoutfuncs]
pub struct InvertedTree {
    pub tree_size: usize,
    pub inverted_list: InvListLblPost,
}

impl InOutFuncs for InvertedTree {
    fn input(input: &core::ffi::CStr) -> Self
    where
        Self: Sized,
    {
        Self::from(parse_tree(input).expect("failed to parse input tree"))
    }

    fn output(&self, buffer: &mut pgrx::StringInfo) {
        let repre = self
            .inverted_list
            .iter()
            .map(|(label, node_count)| format!("{label} -> {node_count}"))
            .collect::<Vec<String>>()
            .join(", ");
        buffer.push_str(repre.as_str());
    }
}

impl From<TreeArena> for InvertedTree {
    fn from(tree: TreeArena) -> Self {
        let mut inverted_list = InvListLblPost::default();
        let Some(root) = tree.iter().next() else {
            panic!("Unable to get root but tree is not empty!");
        };
        let root_id = tree.get_node_id(root).unwrap();
        traverse_inverted(root_id, &tree, &mut inverted_list, 0);

        Self {
            tree_size: tree.count(),
            inverted_list,
        }
    }
}

fn traverse_inverted(
    nid: NodeId,
    tree: &ParsedTree,
    inverted_list: &mut InvListLblPost,
    start_postorder: i32,
) -> i32 {
    let label = tree.get(nid).unwrap().get().clone();
    let mut postorder_id = start_postorder;
    let mut children = 0;
    for cnid in nid.children(tree) {
        postorder_id += traverse_inverted(cnid, tree, inverted_list, postorder_id);
        children += 1;
    }
    inverted_list
        .entry(label)
        .and_modify(|node_count| *node_count += 1)
        .or_insert(1);
    children + 1
}
