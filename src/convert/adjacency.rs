//! Adjacency-list rows `(id, parent_id, label)` to a tree: the core of `tree_agg`.

use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

use thiserror::Error;

use super::BracketWriter;

/// One row of an adjacency list.
pub(crate) struct Row<K> {
    pub id: K,
    pub parent: Option<K>,
    pub label: String,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub(crate) enum AdjacencyError {
    #[error("tree_agg: no root: every row's parent_id is the id of a row in the group")]
    NoRoot,
    #[error("tree_agg: more than one root (ids {0} and {1}): a group must form a single tree")]
    MultipleRoots(String, String),
    #[error("tree_agg: duplicate id {0}")]
    DuplicateId(String),
    #[error("tree_agg: {0} rows are not connected to the root: their parent links form a cycle")]
    Unreachable(usize),
}

/// The tree formed by `rows`, in bracket notation.
///
/// The root is the one row whose parent is `None` or not the id of any row.
/// Children keep the order of `rows`.
pub(crate) fn build<K: Eq + Hash + Display>(rows: &[Row<K>]) -> Result<String, AdjacencyError> {
    let mut index = HashMap::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        if index.insert(&row.id, i).is_some() {
            return Err(AdjacencyError::DuplicateId(row.id.to_string()));
        }
    }

    let mut children = vec![Vec::new(); rows.len()];
    let mut root = None;
    for (i, row) in rows.iter().enumerate() {
        match row.parent.as_ref().and_then(|p| index.get(p)) {
            Some(&parent) => children[parent].push(i),
            None => {
                if let Some(first) = root.replace(i) {
                    let first = rows[first].id.to_string();
                    return Err(AdjacencyError::MultipleRoots(first, row.id.to_string()));
                }
            }
        }
    }
    let root = root.ok_or(AdjacencyError::NoRoot)?;

    // Preorder walk with an explicit stack, so deep trees cannot overflow the
    // call stack. Each entry is a row and the number of its children written.
    let mut out = BracketWriter::new();
    out.open(&rows[root].label);
    let mut written = 1;
    let mut stack = vec![(root, 0)];
    while let Some(top) = stack.last_mut() {
        let (row, next) = *top;
        match children[row].get(next) {
            Some(&child) => {
                top.1 += 1;
                out.open(&rows[child].label);
                written += 1;
                stack.push((child, 0));
            }
            None => {
                out.close();
                stack.pop();
            }
        }
    }

    // Every row has one parent link, so the walk visits each row at most once;
    // rows it misses sit on a cycle of parent links.
    if written < rows.len() {
        return Err(AdjacencyError::Unreachable(rows.len() - written));
    }
    Ok(out.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64, parent: Option<i64>, label: &str) -> Row<i64> {
        Row {
            id,
            parent,
            label: label.to_owned(),
        }
    }

    #[test]
    fn single_row() {
        assert_eq!(build(&[row(1, None, "r")]).unwrap(), "{r}");
    }

    #[test]
    fn children_keep_row_order() {
        let rows = [
            row(1, None, "r"),
            row(3, Some(1), "c"),
            row(2, Some(1), "b"),
        ];
        assert_eq!(build(&rows).unwrap(), "{r{c}{b}}");
    }

    #[test]
    fn nested() {
        let rows = [
            row(1, None, "a"),
            row(2, Some(1), "b"),
            row(3, Some(1), "c"),
            row(4, Some(2), "d"),
        ];
        assert_eq!(build(&rows).unwrap(), "{a{b{d}}{c}}");
    }

    #[test]
    fn rows_may_come_before_their_parent() {
        let rows = [
            row(4, Some(2), "d"),
            row(2, Some(1), "b"),
            row(1, None, "a"),
        ];
        assert_eq!(build(&rows).unwrap(), "{a{b{d}}}");
    }

    #[test]
    fn root_may_point_outside_the_group() {
        // A subtree cut out of a larger hierarchy: its root still has a parent.
        let rows = [row(5, Some(99), "sub"), row(6, Some(5), "x")];
        assert_eq!(build(&rows).unwrap(), "{sub{x}}");
    }

    #[test]
    fn text_ids() {
        let rows = [
            Row {
                id: "b".to_owned(),
                parent: Some("a".to_owned()),
                label: "child".to_owned(),
            },
            Row {
                id: "a".to_owned(),
                parent: None,
                label: "root".to_owned(),
            },
        ];
        assert_eq!(build(&rows).unwrap(), "{root{child}}");
    }

    #[test]
    fn labels_are_escaped() {
        let rows = [row(1, None, "a{"), row(2, Some(1), r"b\")];
        assert_eq!(build(&rows).unwrap(), r"{a\{{b\ }}");
    }

    #[test]
    fn no_root_when_all_rows_form_a_cycle() {
        let rows = [row(1, Some(2), "a"), row(2, Some(1), "b")];
        assert_eq!(build(&rows), Err(AdjacencyError::NoRoot));
    }

    #[test]
    fn more_than_one_root() {
        let rows = [row(1, None, "a"), row(2, None, "b")];
        assert_eq!(
            build(&rows),
            Err(AdjacencyError::MultipleRoots(
                "1".to_owned(),
                "2".to_owned()
            ))
        );
    }

    #[test]
    fn duplicate_id() {
        let rows = [
            row(1, None, "a"),
            row(2, Some(1), "b"),
            row(2, Some(1), "c"),
        ];
        assert_eq!(
            build(&rows),
            Err(AdjacencyError::DuplicateId("2".to_owned()))
        );
    }

    #[test]
    fn cycle_detached_from_the_root() {
        let rows = [
            row(1, None, "r"),
            row(2, Some(3), "a"),
            row(3, Some(2), "b"),
        ];
        assert_eq!(build(&rows), Err(AdjacencyError::Unreachable(2)));
    }

    #[test]
    fn row_that_is_its_own_parent() {
        let rows = [row(1, None, "r"), row(2, Some(2), "a")];
        assert_eq!(build(&rows), Err(AdjacencyError::Unreachable(1)));
    }

    #[test]
    fn deep_chain_does_not_overflow_the_stack() {
        let n = 200_000;
        let rows: Vec<_> = (0..n)
            .map(|i| row(i, if i == 0 { None } else { Some(i - 1) }, "x"))
            .collect();
        let bracket = build(&rows).unwrap();
        assert_eq!(bracket.len(), 3 * n as usize);
        assert!(bracket.starts_with("{x{x") && bracket.ends_with("}}"));
    }
}
