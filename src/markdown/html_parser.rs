//! An html parser used for parsing inline html used in markdown
//!
use html5ever::{
    local_name, namespace_url, ns, parse_document, parse_fragment, tendril::TendrilSink, QualName,
};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use once_cell::sync::Lazy;
use sauron::prelude::*;
use sauron::{
    html,
    html::tags::{HTML_SC_TAGS, HTML_TAGS, HTML_TAGS_NON_COMMON, HTML_TAGS_WITH_MACRO_NON_COMMON},
    svg::tags::{SVG_TAGS, SVG_TAGS_NON_COMMON, SVG_TAGS_SPECIAL},
};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io;
use std::iter::FromIterator;
use thiserror::Error;

/// All of the svg tags
static ALL_SVG_TAGS: Lazy<HashSet<&&'static str>> = Lazy::new(|| {
    HashSet::from_iter(
        SVG_TAGS
            .iter()
            .chain(SVG_TAGS_NON_COMMON.iter())
            .chain(SVG_TAGS_SPECIAL.iter().map(|(_func, t)| t)),
    )
});

/// All of the html tags, excluding the SVG tags.
static ALL_HTML_TAGS: Lazy<HashSet<&&'static str>> = Lazy::new(|| {
    HashSet::from_iter(
        HTML_TAGS
            .iter()
            .chain(HTML_SC_TAGS.iter())
            .chain(HTML_TAGS_NON_COMMON.iter())
            .chain(HTML_TAGS_WITH_MACRO_NON_COMMON.iter()),
    )
});

static ALL_ATTRS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from_iter(
        HTML_ATTRS
            .iter()
            .chain(SVG_ATTRS.iter())
            .map(|att| (*att, *att))
            .chain(
                HTML_ATTRS_SPECIAL
                    .iter()
                    .chain(SVG_ATTRS_SPECIAL.iter())
                    .chain(SVG_ATTRS_XLINK.iter())
                    .map(|(func, att)| (*func, *att)),
            ),
    )
});

static SELF_CLOSING_TAGS: Lazy<HashSet<&&'static str>> =
    Lazy::new(|| HashSet::from_iter(HTML_SC_TAGS.iter()));

/// return the matching attribute
pub fn match_attribute(att: &str) -> Option<&'static str> {
    ALL_ATTRS
        .iter()
        .find(|(_k, v)| v == &&att)
        .map(|(_k, v)| *v)
}

/// return the matching tag
pub fn match_tag(tag: &str) -> Option<&'static str> {
    ALL_HTML_TAGS
        .iter()
        .chain(ALL_SVG_TAGS.iter())
        .find(|t| **t == &tag)
        .map(|t| **t)
}

/// Returns true if this html tag is self closing
#[inline]
pub fn is_self_closing(tag: &str) -> bool {
    SELF_CLOSING_TAGS.contains(&tag)
}

fn extract_attributes<MSG>(attrs: &Vec<html5ever::Attribute>) -> Vec<Attribute<MSG>> {
    attrs
        .iter()
        .filter_map(|att| {
            let key = att.name.local.to_string();
            let value = att.value.to_string();
            if let Some(attr) = match_attribute(&key) {
                Some(html::attributes::attr(attr, value))
            } else {
                log::warn!("Not a standard html attribute: {}", key);
                None
            }
        })
        .collect()
}

fn process_children<MSG>(node: &Handle) -> Vec<Node<MSG>> {
    node.children
        .borrow()
        .iter()
        .filter_map(|child_node| process_node(child_node))
        .collect()
}

fn process_node<MSG>(node: &Handle) -> Option<Node<MSG>> {
    match &node.data {
        NodeData::Text { ref contents } => {
            let text_content = contents.borrow().to_string();
            if text_content.trim().is_empty() {
                None
            } else {
                Some(text(text_content))
            }
        }

        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            let tag = name.local.to_string();
            if let Some(html_tag) = match_tag(&tag) {
                let children_nodes = process_children(node);
                let attributes = extract_attributes(&attrs.borrow());
                let is_self_closing = HTML_SC_TAGS.contains(&html_tag);
                Some(html_element_self_closing(
                    html_tag,
                    attributes,
                    children_nodes,
                    is_self_closing,
                ))
            } else {
                log::warn!("Invalid tag: {}", tag);
                None
            }
        }
        NodeData::Document => {
            let mut children_nodes = process_children(node);
            let children_len = children_nodes.len();
            if children_len == 1 {
                Some(children_nodes.remove(0))
            } else if children_len == 2 {
                Some(children_nodes.remove(1))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// all the possible error when parsing html string
#[derive(Debug, Error)]
pub enum ParseError {
    /// io error
    #[error("{0}")]
    IoError(#[from] io::Error),
    /// formatting error
    #[error("{0}")]
    FmtError(#[from] fmt::Error),
}

/// Parse html string and convert it into sauron Node
fn parse<MSG>(html: &str) -> Result<Option<Node<MSG>>, ParseError> {
    let html_start = html.trim_start();
    let parser = if html_start.starts_with("<html") || html_start.starts_with("<!DOCTYPE") {
        parse_document(RcDom::default(), Default::default())
    } else {
        parse_fragment(
            RcDom::default(),
            Default::default(),
            QualName::new(None, ns!(html), local_name!("div")),
            vec![],
        )
    };

    let dom = parser.one(html);
    let node = process_node(&dom.document);
    Ok(node)
}

/// the document is not wrapped with html
pub fn parse_simple<MSG>(html: &str) -> Result<Vec<Node<MSG>>, ParseError> {
    if let Some(html) = parse(html)? {
        if let Some(element) = html.take_element() {
            assert_eq!(*element.tag(), "html");
            Ok(element.take_children())
        } else {
            Ok(vec![])
        }
    } else {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_match_tags() {
        assert_eq!(Some("div"), match_tag(&String::from("div")));
        assert_eq!(Some("svg"), match_tag(&String::from("svg")));
        assert_eq!(
            Some("color-profile"),
            match_tag(&String::from("color-profile"))
        );
    }

    #[test]
    fn test_html_child() {
        let html = r#"<article class="side-to-side">
    <div>
        This is div content1
    </div>
    <footer>
        This is footer
    </footer>
</article>"#;
        let node: Vec<Node<()>> = parse_simple(html).expect("must parse");
        println!("node: {:#?}", node);
        let one = div(vec![], node);
        println!("one: {}", one.render_to_string());
    }
}
