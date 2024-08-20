use super::{
    super::tree_arena::TreeArena,
    errors::NodeError,
    relations::insert_with_neighbors,
    siblings_range::SiblingsRange,
    traversals::{Ancestors, Descendants, Predecessors, ReverseTraverse, Traverse},
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroUsize;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Copy, Clone, Hash, Deserialize, Serialize)]
pub struct NodeId(NonZeroUsize);

impl NodeId {
    pub fn index(&self) -> usize {
        self.0.get() - 1
    }

    pub fn from_non_zero_usize(index: NonZeroUsize) -> Self {
        NodeId(index)
    }

    /// Returns Ancestors iterator
    pub fn ancestors(self, arena: &TreeArena) -> Ancestors<'_> {
        Ancestors::new(arena, self)
    }

    /// Returns Descendants iterator
    pub fn descendants(self, arena: &TreeArena) -> Descendants<'_> {
        Descendants::new(arena, self)
    }

    pub fn traverse<T>(self, arena: &TreeArena) -> Traverse<'_> {
        Traverse::new(arena, self)
    }

    pub fn predecessors<T>(self, arena: &TreeArena) -> Predecessors<'_> {
        Predecessors::new(arena, self)
    }

    pub fn reverse_traverse<T>(self, arena: &TreeArena) -> ReverseTraverse<'_> {
        ReverseTraverse::new(arena, self)
    }

    /// Appends a new child to this node, after existing children.
    pub fn append(self, new_child: NodeId, arena: &mut TreeArena) {
        self.checked_append(new_child, arena)
            .expect("Preconditions not met: invalid argument");
    }

    /// Detaches a node from its parent and siblings. Children are not affected.
    pub fn detach(self, arena: &mut TreeArena) {
        let range = SiblingsRange::new(self, self).detach_from_siblings(arena);
        range
            .rewrite_parents(arena, None)
            .expect("Should never happen: `None` as parent is always valid");

        // Ensure the node is surely detached.
        debug_assert!(
            arena[self].is_detached(),
            "The node should be successfully detached"
        );
    }

    /// Appends a new child to this node, after existing children.
    pub fn checked_append(self, new_child: NodeId, arena: &mut TreeArena) -> Result<(), NodeError> {
        if new_child == self {
            return Err(NodeError::AppendSelf);
        }
        // if self.ancestors(arena).any(|ancestor| new_child == ancestor) {
        //     return Err(NodeError::AppendAncestor);
        // }

        new_child.detach(arena);
        insert_with_neighbors(arena, new_child, Some(self), arena[self].last_child, None)
            .expect("Should never fail: `new_child` is not `self` and they are not removed");

        Ok(())
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(clippy::from_over_into)]
impl Into<NonZeroUsize> for NodeId {
    fn into(self) -> NonZeroUsize {
        self.0
    }
}

#[allow(clippy::from_over_into)]
impl Into<usize> for NodeId {
    fn into(self) -> usize {
        self.0.get()
    }
}
