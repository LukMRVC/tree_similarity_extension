use super::id::NodeId;
use pgrx::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum NodeData {
    /// Data store
    Data(String),
    /// Next free node position
    NextFree(Option<usize>),
}

#[derive(PostgresType, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Node {
    pub parent: Option<NodeId>,
    pub previous_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,

    pub data: NodeData,
}

impl Node {
    pub fn get(&self) -> &String {
        if let NodeData::Data(ref data) = self.data {
            data
        } else {
            unreachable!("Access to a non-existent node!")
        }
    }

    pub fn get_mut(&mut self) -> &mut String {
        if let NodeData::Data(ref mut data) = self.data {
            data
        } else {
            unreachable!("Access to a non-existent node!")
        }
    }

    pub fn new(data: &str) -> Self {
        Self {
            data: NodeData::Data(data.to_string()),
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
        }
    }

    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    pub fn first_child(&self) -> Option<NodeId> {
        self.first_child
    }

    pub fn last_child(&self) -> Option<NodeId> {
        self.last_child
    }

    pub fn previous_sibling(&self) -> Option<NodeId> {
        self.previous_sibling
    }

    pub fn next_sibling(&self) -> Option<NodeId> {
        self.next_sibling
    }

    /// Checks if the node is detached.
    pub(crate) fn is_detached(&self) -> bool {
        self.parent.is_none() && self.previous_sibling.is_none() && self.next_sibling.is_none()
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "parent: {}; ", parent)?;
        } else {
            write!(f, "no parent; ")?;
        }
        if let Some(previous_sibling) = self.previous_sibling {
            write!(f, "previous sibling: {}; ", previous_sibling)?;
        } else {
            write!(f, "no previous sibling; ")?;
        }
        if let Some(next_sibling) = self.next_sibling {
            write!(f, "next sibling: {}; ", next_sibling)?;
        } else {
            write!(f, "no next sibling; ")?;
        }
        if let Some(first_child) = self.first_child {
            write!(f, "first child: {}; ", first_child)?;
        } else {
            write!(f, "no first child; ")?;
        }
        if let Some(last_child) = self.last_child {
            write!(f, "last child: {}; ", last_child)?;
        } else {
            write!(f, "no last child; ")?;
        }
        Ok(())
    }
}
