//! JSON to tree, after the JSON tree model of JEDI (Hütter, Augsten et al.,
//! SIGMOD 2022): an object is a node `{}` whose children are one node per key,
//! each with the value as its only child; an array is a node `[]` with its
//! elements in order; a literal is a leaf.
//!
//! Object keys are sorted, since JSON objects are unordered and TED is ordered.
//! Strings become their text without quotes, numbers their text as written
//! (serde_json's `arbitrary_precision`), and `true`, `false`, `null` themselves.

use serde_json::Value;

use super::BracketWriter;

/// `value` as a tree in bracket notation.
pub(crate) fn to_bracket(value: &Value) -> String {
    let mut out = BracketWriter::new();
    write(value, &mut out);
    out.finish()
}

/// Recursive: nesting depth is bounded by serde_json's parser (128 levels).
fn write(value: &Value, out: &mut BracketWriter) {
    match value {
        Value::Null => out.leaf("null"),
        Value::Bool(b) => out.leaf(if *b { "true" } else { "false" }),
        Value::Number(n) => out.leaf(&n.to_string()),
        Value::String(s) => out.leaf(s),
        Value::Array(items) => {
            out.open("[]");
            for item in items {
                write(item, out);
            }
            out.close();
        }
        Value::Object(map) => {
            out.open("{}");
            // Sorted here rather than relying on the map's order, which
            // serde_json's `preserve_order` feature would change.
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_unstable_by(|a, b| a.0.cmp(b.0));
            for (key, v) in entries {
                out.open(key);
                write(v, out);
                out.close();
            }
            out.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(json: &str) -> String {
        to_bracket(&serde_json::from_str(json).unwrap())
    }

    #[test]
    fn literals_are_leaves() {
        assert_eq!(tree(r#""hi""#), "{hi}");
        assert_eq!(tree("true"), "{true}");
        assert_eq!(tree("false"), "{false}");
        assert_eq!(tree("null"), "{null}");
        assert_eq!(tree("-3"), "{-3}");
    }

    /// Postgres prints jsonb numbers in plain decimal, never with an exponent.
    #[test]
    fn numbers_keep_their_text() {
        assert_eq!(tree("1.10"), "{1.10}");
        assert_eq!(tree("12345678901234567890123"), "{12345678901234567890123}");
        assert_eq!(
            tree("0.1000000000000000000001"),
            "{0.1000000000000000000001}"
        );
    }

    #[test]
    fn object_keys_are_sorted_and_hold_their_value() {
        assert_eq!(tree(r#"{"b": 1, "a": 2}"#), r"{\{\}{a{2}}{b{1}}}");
    }

    #[test]
    fn keys_sort_by_bytes() {
        assert_eq!(
            tree(r#"{"b": 1, "aa": 1, "a": 1, "B": 1}"#),
            r"{\{\}{B{1}}{a{1}}{aa{1}}{b{1}}}"
        );
    }

    #[test]
    fn arrays_keep_their_order() {
        assert_eq!(tree("[3, 1, 2]"), "{[]{3}{1}{2}}");
    }

    #[test]
    fn nested() {
        assert_eq!(
            tree(r#"{"a": [1, "x"], "b": {"c": null}}"#),
            r"{\{\}{a{[]{1}{x}}}{b{\{\}{c{null}}}}}"
        );
    }

    #[test]
    fn empty_containers() {
        assert_eq!(tree("{}"), r"{\{\}}");
        assert_eq!(tree("[]"), "{[]}");
        assert_eq!(tree(r#"{"": ""}"#), r"{\{\}{{}}}");
    }

    #[test]
    fn strings_are_unescaped() {
        assert_eq!(tree(r#""a\nb""#), "{a\nb}");
        assert_eq!(tree(r#""ü\"""#), "{ü\"}");
    }

    #[test]
    fn braces_in_strings_and_keys_are_escaped() {
        assert_eq!(tree(r#"{"{k}": "{v}"}"#), r"{\{\}{\{k\}{\{v\}}}}");
    }
}
