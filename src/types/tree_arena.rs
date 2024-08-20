use super::tree_internals::{node::Node, id::NodeId};
use pgrx::prelude::*;
use serde::*;
use std::{ops::{Index, IndexMut}, slice, mem, num::NonZeroUsize, fmt};
use std::fmt::Formatter;
use crate::parsing::parse_tree;
use crate::types::tree_internals::traversals::NodeEdge;

#[derive(PostgresType, Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[inoutfuncs]
pub struct TreeArena {
    nodes: Vec<Node>,
    first_free_slot: Option<usize>,
    last_free_slot: Option<usize>,
}

impl InOutFuncs for TreeArena {
    fn input(input: &core::ffi::CStr) -> Self
    where
        Self: Sized,
    {
        parse_tree(input).unwrap()
    }

    fn output(&self, buffer: &mut pgrx::StringInfo) {
        buffer.push_str(&self.to_string())
    }
}

impl TreeArena {
    /// Creates a new empty `TreeArena` with enough capacity to store `n` nodes.
    pub fn new() -> Self  {
        Self::default()
    }

    /// Creates a new empty `TreeArena` with enough capacity to store `n` nodes.
    pub fn with_capacity(n: usize) -> Self  {
        Self {
            nodes: Vec::with_capacity(n),
            ..Self::default()
        }
    }

    /// Returns the number of nodes the arena can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    pub fn get_root_id(&self) -> Option<NodeId> {
        if self.nodes.is_empty() {
            None
        } else {
            let node_id = NonZeroUsize::new(1)?;
            Some(NodeId::from_non_zero_usize(node_id))
        }
    }

    pub fn get_root(&self) -> Option<&Node> {
        if self.nodes.is_empty() {
            None
        } else {
            self.nodes.get(0)
        }
    }

    pub fn get_root_mut(&mut self) -> Option<&mut Node> {
        if self.nodes.is_empty() {
            None
        } else {
            self.nodes.get_mut(0)
        }
    }

    /// Retrieves the `NodeId` corresponding to a `Node` in the `Arena`.
    pub fn get_node_id(&self, node: &Node) -> Option<NodeId> {
        let nodes_range = self.nodes.as_ptr_range();
        let p = node as *const Node;

        if !nodes_range.contains(&p) {
            return None;
        }

        let node_index = (p as usize - nodes_range.start as usize) / mem::size_of::<Node>();
        let node_id = NonZeroUsize::new(node_index.wrapping_add(1))?;

        Some(NodeId::from_non_zero_usize(
            node_id,
        ))
    }

    /// Creates a new node from its associated data.
    ///
    /// # Panics
    ///
    /// Panics if the arena already has `usize::max_value()` nodes.
    pub fn new_node(&mut self, data: String) -> NodeId {
        let index = {
            let index = self.nodes.len();
            let node = Node::new(&data);
            self.nodes.push(node);
            index
        };
        let next_index1 =
            NonZeroUsize::new(index.wrapping_add(1)).expect("Too many nodes in the arena");
        NodeId::from_non_zero_usize(next_index1)
    }

    /// Counts the number of nodes in arena and returns it.
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if arena has no nodes, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Returns a reference to the node with the given id if in the arena.
    ///
    /// Returns `None` if not available.
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.index())
    }

    /// Returns a mutable reference to the node with the given id if in the
    /// arena.
    ///
    /// Returns `None` if not available.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.index())
    }


    /// Returns an iterator of all nodes in the arena in storage-order.
    pub fn iter(&self) -> slice::Iter<Node> {
        self.nodes.iter()
    }

    /// Returns a mutable iterator of all nodes in the arena in storage-order.
    pub fn iter_mut(&mut self) -> slice::IterMut<Node> {
        self.nodes.iter_mut()
    }

    /// Clears all the nodes in the arena, but retains its allocated capacity.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.first_free_slot = None;
        self.last_free_slot = None;
    }

    /// Returns a slice of the inner nodes collection.
    ///
    /// Note that this **does not** return root elements, it simply
    /// returns a slice into the internal representation of the arena.
    pub fn as_slice(&self) -> &[Node] {
        self.nodes.as_slice()
    }
}

impl Index<NodeId> for TreeArena {
    type Output = Node;

    fn index(&self, node: NodeId) -> &Node {
        &self.nodes[node.index()]
    }
}

impl IndexMut<NodeId> for TreeArena {
    fn index_mut(&mut self, node: NodeId) -> &mut Node {
        &mut self.nodes[node.index()]
    }
}

impl fmt::Display for TreeArena {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut bracket_notation = String::with_capacity(self.count() * 2);
        let Some(root) = self.iter().next() else {
            panic!("Root not found!");
        };
        let root_id = self.get_node_id(root).expect("Root ID not found!");

        for edge in root_id.traverse(self) {
            match edge {
                NodeEdge::Start(node_id) => {
                    bracket_notation.push('{');
                    bracket_notation.push_str(self.get(node_id).unwrap().get());
                }
                NodeEdge::End(_) => {
                    bracket_notation.push('}');
                }
            }
        }

        write!(f, "{}", bracket_notation)
    }
}