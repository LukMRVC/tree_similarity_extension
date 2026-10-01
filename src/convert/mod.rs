//! Building `tree` values from other encodings of a hierarchy: adjacency-list
//! rows (`tree_agg`), JSON documents (`tree_from_jsonb`) and XML documents
//! (`tree_from_xml`).
//!
//! Every converter emits bracket notation through [`BracketWriter`], which
//! escapes labels so that any string can be a label.

mod adjacency;
mod json;
mod xml;

use std::fmt::Display;
use std::hash::Hash;

use pgrx::prelude::*;
use pgrx::{name, AggregateName, Internal, JsonB};

use crate::types::Tree;
use adjacency::Row;

fn to_tree(bracket: String) -> Tree {
    Tree::parse(&bracket).unwrap_or_else(|e| error!("built an invalid tree {bracket:?}: {e}"))
}

#[pg_extern(immutable, parallel_safe)]
fn tree_from_jsonb(doc: JsonB) -> Tree {
    to_tree(json::to_bracket(&doc.0))
}

#[pg_extern(immutable, parallel_safe)]
fn tree_from_xml(doc: &str) -> Tree {
    xml::to_bracket(doc)
        .map(to_tree)
        .unwrap_or_else(|e| error!("{e}"))
}

/// `tree_agg(id, parent_id, label)` over `bigint` ids.
#[derive(Copy, Clone, Default, Debug, AggregateName)]
#[aggregate_name = "tree_agg"]
pub struct TreeAggBigint;

#[pg_aggregate]
impl Aggregate<TreeAggBigint> for TreeAggBigint {
    type State = Internal;
    type Args = (
        name!(id, Option<i64>),
        name!(parent_id, Option<i64>),
        name!(label, Option<String>),
    );
    type Finalize = Option<Tree>;

    fn state(
        mut current: Self::State,
        (id, parent_id, label): Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
        push_row(&mut current, id, parent_id, label);
        current
    }

    fn finalize(
        mut current: Self::State,
        _direct_args: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        build_tree::<i64>(&mut current)
    }
}

/// `tree_agg(id, parent_id, label)` over `text` ids.
#[derive(Copy, Clone, Default, Debug, AggregateName)]
#[aggregate_name = "tree_agg"]
pub struct TreeAggText;

#[pg_aggregate]
impl Aggregate<TreeAggText> for TreeAggText {
    type State = Internal;
    type Args = (
        name!(id, Option<String>),
        name!(parent_id, Option<String>),
        name!(label, Option<String>),
    );
    type Finalize = Option<Tree>;

    fn state(
        mut current: Self::State,
        (id, parent_id, label): Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
        push_row(&mut current, id, parent_id, label);
        current
    }

    fn finalize(
        mut current: Self::State,
        _direct_args: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        build_tree::<String>(&mut current)
    }
}

/// Transition step shared by both `tree_agg`s: collect the row.
fn push_row<K: Display + 'static>(
    state: &mut Internal,
    id: Option<K>,
    parent: Option<K>,
    label: Option<String>,
) {
    let Some(id) = id else {
        error!("tree_agg: id must not be NULL")
    };
    let Some(label) = label else {
        error!("tree_agg: label must not be NULL (id {id})")
    };
    // SAFETY: this aggregate's state only ever holds a `Vec<Row<K>>`.
    unsafe { state.get_or_insert_default::<Vec<Row<K>>>() }.push(Row { id, parent, label });
}

/// Final step shared by both `tree_agg`s. No rows (no state) give NULL.
fn build_tree<K: Eq + Hash + Display + 'static>(state: &mut Internal) -> Option<Tree> {
    // SAFETY: as in `push_row`.
    let rows = unsafe { state.get_mut::<Vec<Row<K>>>() }?;
    Some(
        adjacency::build(rows)
            .map(to_tree)
            .unwrap_or_else(|e| error!("{e}")),
    )
}

/// Writes bracket notation node by node.
///
/// The parser treats a brace preceded by `\` as part of the label and keeps the
/// `\`, so a `\` is written before every `{` and `}` of a label. A label ending
/// in `\` would escape the brace after it, so a space is appended to it; that
/// is the one case where two labels (`x\` and `x\ `) end up equal.
pub(crate) struct BracketWriter {
    out: String,
}

impl BracketWriter {
    pub fn new() -> Self {
        Self { out: String::new() }
    }

    /// Opens a node; its children follow until the matching [`close`](Self::close).
    pub fn open(&mut self, label: &str) {
        self.out.push('{');
        for c in label.chars() {
            if c == '{' || c == '}' {
                self.out.push('\\');
            }
            self.out.push(c);
        }
        if label.ends_with('\\') {
            self.out.push(' ');
        }
    }

    pub fn close(&mut self) {
        self.out.push('}');
    }

    /// A node without children.
    pub fn leaf(&mut self, label: &str) {
        self.open(label);
        self.close();
    }

    pub fn finish(self) -> String {
        self.out
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::parsing::{walk_bracket, BracketVisitor};

    /// The labels of `bracket` in preorder, as the parser sees them.
    pub(crate) fn parsed_labels(bracket: &str) -> Vec<String> {
        struct Labels(Vec<String>);
        impl BracketVisitor for Labels {
            fn open(&mut self, label: &[u8]) {
                self.0.push(String::from_utf8(label.to_vec()).unwrap());
            }
            fn close(&mut self) {}
        }
        let mut labels = Labels(Vec::new());
        walk_bracket(bracket.as_bytes(), &mut labels).expect("writer output must parse");
        labels.0
    }

    fn leaf(label: &str) -> String {
        let mut w = BracketWriter::new();
        w.leaf(label);
        w.finish()
    }

    #[test]
    fn plain_label_is_written_as_is() {
        assert_eq!(leaf("a b"), "{a b}");
        assert_eq!(leaf(""), "{}");
    }

    #[test]
    fn nested_nodes() {
        let mut w = BracketWriter::new();
        w.open("a");
        w.leaf("b");
        w.open("c");
        w.leaf("d");
        w.close();
        w.close();
        assert_eq!(w.finish(), "{a{b}{c{d}}}");
    }

    #[test]
    fn braces_in_labels_are_escaped() {
        assert_eq!(leaf("a{b}c"), r"{a\{b\}c}");
        assert_eq!(parsed_labels(&leaf("a{b}c")), [r"a\{b\}c"]);
        assert_eq!(parsed_labels(&leaf("{}")), [r"\{\}"]);
    }

    #[test]
    fn label_ending_in_backslash_gets_a_space() {
        assert_eq!(leaf(r"a\"), r"{a\ }");
        assert_eq!(parsed_labels(&leaf(r"\")), [r"\ "]);
    }

    #[test]
    fn escaped_labels_stay_distinct() {
        // Escaping only inserts `\` before braces, so it is injective.
        assert_ne!(leaf("a{"), leaf(r"a\{"));
        assert_eq!(parsed_labels(&leaf(r"a\{")), [r"a\\{"]);
    }

    #[test]
    fn awkward_labels_keep_the_tree_shape() {
        let labels = ["{", "}", r"\", r"\{", r"x}\", "", "a b"];
        let mut w = BracketWriter::new();
        w.open("root");
        for l in labels {
            w.leaf(l);
        }
        w.close();
        let parsed = parsed_labels(&w.finish());
        assert_eq!(parsed.len(), labels.len() + 1, "{parsed:?}");
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pgrx::pg_schema]
mod tests {
    use pgrx::prelude::*;

    /// A scalar query's single text result.
    fn text(query: &str) -> Option<String> {
        Spi::get_one::<String>(query).unwrap()
    }

    fn tree_text(expr: &str) -> String {
        text(&format!("SELECT ({expr})::text")).expect("non-NULL tree")
    }

    /// `nodes(id, parent_id, label, pos)`, the table the docs use. Siblings are
    /// out of id order, one `pos` is tied (so `id` breaks it), and the labels
    /// need escaping.
    fn create_nodes() {
        Spi::run(
            r"CREATE TABLE nodes (id int PRIMARY KEY, parent_id int, label text, pos int);
              INSERT INTO nodes VALUES
                (1, NULL, 'root', 1),
                (5, 1, 'a{b', 1),
                (2, 1, 'c}', 2),
                (3, 1, 'd\', 2),
                (4, 5, '\{', 1),
                (6, 5, '', 0),
                (7, 4, 'with space', 1);",
        )
        .unwrap();
    }

    const NODES_TREE: &str = r"{root{a\{b{}{\\{{with space}}}{c\}}{d\ }}";

    #[pg_test]
    fn tree_agg_builds_the_tree() {
        create_nodes();
        assert_eq!(
            tree_text("SELECT tree_agg(id, parent_id, label ORDER BY pos, id) FROM nodes"),
            NODES_TREE
        );
    }

    #[pg_test]
    fn tree_agg_sibling_order_follows_order_by() {
        let rows = "(VALUES (1, NULL, 'r', 1), (2, 1, 'a', 1), (3, 1, 'b', 2)) v(id, parent_id, label, pos)";
        assert_eq!(
            tree_text(&format!(
                "SELECT tree_agg(id, parent_id, label ORDER BY pos) FROM {rows}"
            )),
            "{r{a}{b}}"
        );
        assert_eq!(
            tree_text(&format!(
                "SELECT tree_agg(id, parent_id, label ORDER BY pos DESC) FROM {rows}"
            )),
            "{r{b}{a}}"
        );
    }

    #[pg_test]
    fn tree_agg_one_tree_per_group() {
        let trees = Spi::get_one::<Vec<String>>(
            "SELECT array_agg(t::text ORDER BY doc) FROM (
               SELECT doc, tree_agg(id, parent_id, label ORDER BY id) AS t
               FROM (VALUES (1, 1, NULL, 'x'), (1, 2, 1, 'y'), (2, 1, NULL, 'z')) v(doc, id, parent_id, label)
               GROUP BY doc) s",
        )
        .unwrap()
        .unwrap();
        assert_eq!(trees, ["{x{y}}", "{z}"]);
    }

    #[pg_test]
    fn tree_agg_subtree_by_where() {
        create_nodes();
        assert_eq!(
            tree_text("SELECT tree_agg(id, parent_id, label ORDER BY pos, id) FROM nodes WHERE id IN (4, 7)"),
            r"{\\{{with space}}"
        );
    }

    #[pg_test]
    fn tree_agg_text_ids() {
        assert_eq!(
            tree_text(
                "SELECT tree_agg(id::text, parent_id::text, label ORDER BY label) FROM (VALUES
                   ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'::uuid, NULL::uuid, 'r'),
                   ('b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'::uuid, 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'::uuid, 'c')
                 ) v(id, parent_id, label)"
            ),
            "{r{c}}"
        );
    }

    #[pg_test]
    fn tree_agg_bigint_ids() {
        assert_eq!(
            tree_text("SELECT tree_agg(id, parent_id, 'n' || id) FROM (VALUES (9000000000::bigint, NULL::bigint)) v(id, parent_id)"),
            "{n9000000000}"
        );
    }

    #[pg_test]
    fn tree_agg_of_no_rows_is_null() {
        create_nodes();
        assert_eq!(
            text("SELECT tree_agg(id, parent_id, label)::text FROM nodes WHERE false"),
            None
        );
    }

    #[pg_test(
        error = "tree_agg: more than one root (ids 1 and 2): a group must form a single tree"
    )]
    fn tree_agg_rejects_two_roots() {
        tree_text("SELECT tree_agg(id, parent_id, 'x' ORDER BY id) FROM (VALUES (1, NULL::int), (2, NULL)) v(id, parent_id)");
    }

    #[pg_test(error = "tree_agg: no root: every row's parent_id is the id of a row in the group")]
    fn tree_agg_rejects_no_root() {
        tree_text(
            "SELECT tree_agg(id, parent_id, 'x') FROM (VALUES (1, 2), (2, 1)) v(id, parent_id)",
        );
    }

    #[pg_test(error = "tree_agg: duplicate id 2")]
    fn tree_agg_rejects_duplicate_ids() {
        tree_text("SELECT tree_agg(id, parent_id, 'x' ORDER BY id) FROM (VALUES (1, NULL), (2, 1), (2, 1)) v(id, parent_id)");
    }

    #[pg_test(
        error = "tree_agg: 2 rows are not connected to the root: their parent links form a cycle"
    )]
    fn tree_agg_rejects_cycles() {
        tree_text("SELECT tree_agg(id, parent_id, 'x') FROM (VALUES (1, NULL), (2, 3), (3, 2)) v(id, parent_id)");
    }

    #[pg_test(error = "tree_agg: id must not be NULL")]
    fn tree_agg_rejects_null_id() {
        tree_text("SELECT tree_agg(id, parent_id, 'x') FROM (VALUES (NULL::int, NULL::int)) v(id, parent_id)");
    }

    #[pg_test(error = "tree_agg: label must not be NULL (id 1)")]
    fn tree_agg_rejects_null_label() {
        tree_text("SELECT tree_agg(1, NULL, NULL::text)");
    }

    /// The plain-SQL recipe in docs/usage.md, run as written.
    fn docs_recipe() -> &'static str {
        const DOCS: &str = include_str!("../../docs/usage.md");
        const START: &str = "<!-- recipe:adjacency -->\n```sql\n";
        let start = DOCS.find(START).expect("recipe start marker") + START.len();
        let end = DOCS
            .find("```\n<!-- /recipe:adjacency -->")
            .expect("recipe end marker");
        &DOCS[start..end]
    }

    #[pg_test]
    fn docs_recipe_matches_tree_agg() {
        create_nodes();
        let recipe = tree_text(&format!("SELECT tree FROM ({}) r", docs_recipe()));
        assert_eq!(recipe, NODES_TREE);
        assert_eq!(
            recipe,
            tree_text("SELECT tree_agg(id, parent_id, label ORDER BY pos, id) FROM nodes")
        );
    }

    #[pg_test]
    fn tree_from_jsonb_maps_the_document() {
        assert_eq!(
            tree_text(r#"tree_from_jsonb('{"b": [1, 2.50, true], "a": "x{"}')"#),
            r"{\{\}{a{x\{}}{b{[]{1}{2.50}{true}}}}"
        );
    }

    #[pg_test]
    fn tree_from_jsonb_of_null_is_null() {
        assert_eq!(text("SELECT tree_from_jsonb(NULL)::text"), None);
    }

    #[pg_test]
    fn tree_from_xml_maps_the_document() {
        assert_eq!(
            tree_text(
                r#"tree_from_xml('<article key="a/b" mdate="2012"><title>On <i>trees</i></title></article>')"#
            ),
            "{article{key{a/b}}{mdate{2012}}{title{On }{i{trees}}}}"
        );
    }

    #[pg_test(error = "invalid XML: no root element")]
    fn tree_from_xml_rejects_a_document_without_root() {
        tree_text("tree_from_xml('<!-- nothing -->')");
    }

    #[pg_test]
    fn built_trees_work_with_similarity_search() {
        let within = |k: i32| {
            Spi::get_one::<bool>(&format!(
                r#"SELECT tree_from_jsonb('{{"a": 1}}') <~ tree_query(tree_from_jsonb('{{"a": 2}}'), {k})"#
            ))
            .unwrap()
            .unwrap()
        };
        assert!(within(1));
        assert!(!within(0));
    }
}
