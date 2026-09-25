use std::ffi::CString;

use pgrx::prelude::*;
use pgrx::{InOutFuncs, PostgresType, StringInfo};
use serde::{Deserialize, Serialize};

use crate::parsing::{parse_tree, walk_bracket, BracketVisitor, TreeParseError};
use crate::pipelines::Lb;
use crate::types::{TreeArena, UnifiedTreeIndex};

/// Base tree type: the tree as its bracket-notation text, labels kept as strings.
#[derive(Debug, Clone, PartialEq, Eq, PostgresType, Serialize, Deserialize)]
#[inoutfuncs]
pub struct Tree {
    bracket: String,
}

/// Checks the bracket syntax without building anything.
struct Validate;

impl BracketVisitor for Validate {
    fn open(&mut self, _label: &[u8]) {}
    fn close(&mut self) {}
}

impl Tree {
    pub fn parse(bracket: &str) -> Result<Self, TreeParseError> {
        if bracket.contains('\0') {
            return Err(TreeParseError::IncorrectFormat("tree contains a NUL byte".to_owned()));
        }
        walk_bracket(bracket.as_bytes(), &mut Validate)?;
        Ok(Self { bracket: bracket.to_owned() })
    }

    pub fn as_str(&self) -> &str {
        &self.bracket
    }

    pub fn to_arena(&self) -> TreeArena {
        parse_tree(&self.c_string()).expect("tree was validated on input")
    }

    pub fn to_unified(&self) -> UnifiedTreeIndex {
        UnifiedTreeIndex::parse(&self.c_string()).expect("tree was validated on input")
    }

    fn c_string(&self) -> CString {
        CString::new(self.bracket.as_bytes()).expect("tree was validated on input")
    }
}

impl InOutFuncs for Tree {
    fn input(input: &core::ffi::CStr) -> Self {
        let text = input.to_str().unwrap_or_else(|_| error!("tree input is not valid UTF-8"));
        Tree::parse(text).unwrap_or_else(|e| error!("invalid tree {text:?}: {e}"))
    }

    fn output(&self, buffer: &mut StringInfo) {
        buffer.push_str(&self.bracket);
    }
}

/// Right-hand side of `<~`: a query tree plus the edit-distance threshold `k`.
#[derive(Debug, Clone, PartialEq, Eq, PostgresType, Serialize, Deserialize)]
pub struct TreeQuery {
    pub tree: Tree,
    pub k: i32,
}

#[pg_extern(immutable, parallel_safe)]
fn tree_query(tree: Tree, k: i32) -> TreeQuery {
    TreeQuery { tree, k }
}

/// `tree <~ tree_query(q, k)`: true iff TED(tree, q) <= k.
#[pg_operator(immutable, parallel_safe, cost = 5000)]
#[opname(<~)]
#[restrict(contsel)]
#[join(contjoinsel)]
fn tree_within(tree: Tree, query: TreeQuery) -> bool {
    let k = query.k;
    Lb::SedStruct.within(&query.tree.to_unified(), &tree.to_unified(), k) <= k
}
