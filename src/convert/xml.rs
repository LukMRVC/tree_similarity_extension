//! XML to tree, in the layout of the XML-derived datasets (dblp, swissprot):
//!
//! - an element is a node labelled with its name as written (`x:tag`);
//! - its attributes come first, sorted by name, each a node `{name{value}}`;
//!   namespace declarations (`xmlns`, `xmlns:x`) are attributes too;
//! - then its content in document order: child elements, and text as leaves.
//!
//! Text is kept as written, surrounding whitespace included; text that is only
//! whitespace is dropped. Text, CDATA and references next to each other form
//! one leaf. Comments, processing instructions and the doctype are dropped.
//! Predefined entities and character references are decoded; other entities
//! (declared in a DTD, e.g. `&uuml;`) are kept as written.

use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use thiserror::Error;

use super::BracketWriter;

#[derive(Error, Debug, PartialEq, Eq)]
pub(crate) enum XmlError {
    #[error("invalid XML: no root element")]
    NoRoot,
    #[error("invalid XML: more than one root element")]
    MultipleRoots,
    #[error("invalid XML: text outside the root element")]
    TextOutsideRoot,
    #[error("invalid XML: element <{0}> is not closed")]
    Unclosed(String),
    #[error("invalid XML: {0}")]
    Syntax(String),
}

/// The XML document `doc` as a tree in bracket notation.
pub(crate) fn to_bracket(doc: &str) -> Result<String, XmlError> {
    let mut reader = Reader::from_str(doc);
    let mut out = BracketWriter::new();
    // Names of the elements open around the current position.
    let mut open: Vec<String> = Vec::new();
    let mut seen_root = false;
    // The text read since the last tag; it becomes one leaf.
    let mut text = String::new();

    loop {
        let event = reader
            .read_event()
            .map_err(|e| XmlError::Syntax(format!("{e} (at byte {})", reader.error_position())))?;
        match event {
            Event::Start(e) => {
                flush_text(&mut text, open.is_empty(), &mut out)?;
                start_element(&e, &mut seen_root, open.is_empty(), &mut out)?;
                open.push(e.name().as_ref().to_owned());
            }
            Event::Empty(e) => {
                flush_text(&mut text, open.is_empty(), &mut out)?;
                start_element(&e, &mut seen_root, open.is_empty(), &mut out)?;
                out.close();
            }
            Event::End(_) => {
                flush_text(&mut text, false, &mut out)?;
                out.close();
                open.pop();
            }
            Event::Text(t) => text.push_str(&t.xml_content(XmlVersion::Implicit1_0)),
            Event::CData(t) => text.push_str(&t.xml_content(XmlVersion::Implicit1_0)),
            Event::GeneralRef(r) => {
                if open.is_empty() {
                    return Err(XmlError::TextOutsideRoot);
                }
                push_reference(&r, &mut text)?;
            }
            Event::Comment(_) | Event::PI(_) | Event::Decl(_) | Event::DocType(_) => {}
            Event::Eof => break,
        }
    }

    if let Some(name) = open.pop() {
        return Err(XmlError::Unclosed(name));
    }
    flush_text(&mut text, true, &mut out)?;
    if !seen_root {
        return Err(XmlError::NoRoot);
    }
    Ok(out.finish())
}

/// Opens the node of element `e` and writes its attributes.
fn start_element(
    e: &BytesStart,
    seen_root: &mut bool,
    top_level: bool,
    out: &mut BracketWriter,
) -> Result<(), XmlError> {
    if top_level {
        if *seen_root {
            return Err(XmlError::MultipleRoots);
        }
        *seen_root = true;
    }

    let mut attributes = Vec::new();
    for attr in e.attributes() {
        let attr = attr.map_err(|err| XmlError::Syntax(err.to_string()))?;
        attributes.push((attr.key.as_ref().to_owned(), attribute_value(&attr.value)?));
    }
    attributes.sort_unstable();

    out.open(e.name().as_ref());
    for (name, value) in &attributes {
        out.open(name);
        out.leaf(value);
        out.close();
    }
    Ok(())
}

/// Writes the pending text as a leaf, unless it is only whitespace.
fn flush_text(text: &mut String, top_level: bool, out: &mut BracketWriter) -> Result<(), XmlError> {
    if !text.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) {
        if top_level {
            return Err(XmlError::TextOutsideRoot);
        }
        out.leaf(text);
    }
    text.clear();
    Ok(())
}

/// Appends what the reference `&name;` stands for, or the reference itself
/// when it names an entity only a DTD could define.
fn push_reference(r: &BytesRef, out: &mut String) -> Result<(), XmlError> {
    if let Some(c) = r
        .resolve_char_ref()
        .map_err(|e| XmlError::Syntax(e.to_string()))?
    {
        out.push(c);
    } else if let Some(s) = resolve_predefined_entity(r) {
        out.push_str(s);
    } else {
        out.push('&');
        out.push_str(r);
        out.push(';');
    }
    Ok(())
}

/// A raw attribute value, normalized as XML 1.0 §3.3.3 says (each line break,
/// tab or `\r\n` becomes a space) and with its references resolved.
fn attribute_value(raw: &str) -> Result<String, XmlError> {
    let mut value = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(i) = rest.find(['&', '\t', '\n', '\r']) {
        value.push_str(&rest[..i]);
        rest = &rest[i..];
        if rest.starts_with('&') {
            let end = rest.find(';').ok_or_else(|| {
                XmlError::Syntax(format!("unterminated reference in attribute value {raw:?}"))
            })?;
            push_reference(&BytesRef::new(&rest[1..end]), &mut value)?;
            rest = &rest[end + 1..];
        } else {
            value.push(' ');
            rest = &rest[if rest.starts_with("\r\n") { 2 } else { 1 }..];
        }
    }
    value.push_str(rest);
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(doc: &str) -> String {
        to_bracket(doc).unwrap_or_else(|e| panic!("{doc:?}: {e}"))
    }

    #[test]
    fn element_with_text() {
        assert_eq!(tree("<a>hi</a>"), "{a{hi}}");
    }

    #[test]
    fn empty_elements() {
        assert_eq!(tree("<a/>"), "{a}");
        assert_eq!(tree("<a></a>"), "{a}");
    }

    #[test]
    fn attributes_come_first_sorted_by_name() {
        assert_eq!(tree(r#"<e z="1" a="2"><c/></e>"#), "{e{a{2}}{z{1}}{c}}");
    }

    /// A dblp record, as in dblp.xml, gives exactly the dataset's tree.
    #[test]
    fn dblp_record_matches_the_dataset() {
        let doc = r#"<article mdate="2012-09-19" key="journals/firstmonday/OLeary12">
<author>Zachary O'Leary</author>
<title>Book review of <i>Studying mobile media: Cultural technologies, mobile communication, and the iPhone</i>.</title>
<year>2012</year>
<volume>17</volume>
<journal>First Monday</journal>
<number>9</number>
<url>db/journals/firstmonday/firstmonday17.html#OLeary12</url>
</article>"#;
        assert_eq!(
            tree(doc),
            "{article{key{journals/firstmonday/OLeary12}}{mdate{2012-09-19}}{author{Zachary O'Leary}}\
             {title{Book review of }{i{Studying mobile media: Cultural technologies, mobile communication, \
             and the iPhone}}{.}}{year{2012}}{volume{17}}{journal{First Monday}}{number{9}}\
             {url{db/journals/firstmonday/firstmonday17.html#OLeary12}}}"
        );
    }

    #[test]
    fn namespace_declarations_are_attributes() {
        assert_eq!(
            tree(
                r#"<entry xmlns="http://uniprot.org/uniprot" dataset="Swiss-Prot"><accession>P80438</accession></entry>"#
            ),
            "{entry{dataset{Swiss-Prot}}{xmlns{http://uniprot.org/uniprot}}{accession{P80438}}}"
        );
    }

    #[test]
    fn prefixed_names_are_kept() {
        assert_eq!(
            tree(r#"<x:a xmlns:x="u"><x:b/></x:a>"#),
            "{x:a{xmlns:x{u}}{x:b}}"
        );
    }

    #[test]
    fn whitespace_only_text_is_dropped() {
        assert_eq!(tree("<a>\n  <b/>\n  <c> </c>\n</a>"), "{a{b}{c}}");
    }

    #[test]
    fn text_is_kept_as_written() {
        assert_eq!(tree("<a> x <b/>y </a>"), "{a{ x }{b}{y }}");
    }

    #[test]
    fn text_cdata_and_references_form_one_leaf() {
        assert_eq!(tree("<a>x<![CDATA[<y>]]>&amp;z</a>"), "{a{x<y>&z}}");
    }

    #[test]
    fn comments_pis_and_doctype_are_dropped() {
        let doc =
            r#"<?xml version="1.0"?><!DOCTYPE a><!-- c --><a>x<!-- c -->y<?pi z?></a><!-- end -->"#;
        assert_eq!(tree(doc), "{a{xy}}");
    }

    #[test]
    fn predefined_entities_and_char_refs_are_decoded() {
        assert_eq!(
            tree("<a>&lt;&gt;&amp;&apos;&quot;&#65;&#x42;</a>"),
            "{a{<>&'\"AB}}"
        );
    }

    #[test]
    fn unknown_entities_are_kept() {
        assert_eq!(tree("<a>M&uuml;ller</a>"), "{a{M&uuml;ller}}");
    }

    #[test]
    fn attribute_values_are_decoded_and_normalized() {
        assert_eq!(tree("<a t=\"x&amp;y&#65;&foo;\"/>"), "{a{t{x&yA&foo;}}}");
        assert_eq!(tree("<a t=\"x\ny\tz\"/>"), "{a{t{x y z}}}");
    }

    #[test]
    fn braces_are_escaped() {
        assert_eq!(tree(r#"<a k="{v}">{x}</a>"#), r"{a{k{\{v\}}}{\{x\}}}");
    }

    #[test]
    fn no_root() {
        assert_eq!(to_bracket(""), Err(XmlError::NoRoot));
        assert_eq!(to_bracket("<!-- only a comment -->"), Err(XmlError::NoRoot));
    }

    #[test]
    fn more_than_one_root() {
        assert_eq!(to_bracket("<a/><b/>"), Err(XmlError::MultipleRoots));
    }

    #[test]
    fn text_outside_the_root() {
        assert_eq!(to_bracket("x<a/>"), Err(XmlError::TextOutsideRoot));
        assert_eq!(to_bracket("<a/>x"), Err(XmlError::TextOutsideRoot));
    }

    #[test]
    fn unclosed_element() {
        assert_eq!(
            to_bracket("<a><b></b>"),
            Err(XmlError::Unclosed("a".to_owned()))
        );
    }

    #[test]
    fn syntax_errors() {
        for doc in ["<a></b>", "<a>x & y</a>", "<a", "</a>"] {
            assert!(
                matches!(to_bracket(doc), Err(XmlError::Syntax(_))),
                "{doc:?}: {:?}",
                to_bracket(doc)
            );
        }
    }
}
