//! An html parser used for parsing inline html used in markdown
//!
use once_cell::sync::Lazy;
use rphtml::config::ParseOptions;
use rphtml::parser::Doc;
use rphtml::parser::NodeType;
use rphtml::types::BoxDynError;
use sauron::prelude::*;
use sauron::{
    html::tags::{HTML_SC_TAGS, HTML_TAGS, HTML_TAGS_NON_COMMON, HTML_TAGS_WITH_MACRO_NON_COMMON},
    svg::tags::{SVG_TAGS, SVG_TAGS_NON_COMMON, SVG_TAGS_SPECIAL},
};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io;
use std::iter::FromIterator;
use std::ops::Deref;
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

/// given the attribute return the function name
pub fn attribute_function(att: &str) -> Option<&'static str> {
    ALL_ATTRS
        .iter()
        .find(|(_k, v)| v == &&att)
        .map(|(k, _v)| *k)
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

/// all the possible error when parsing html string
#[derive(Debug, Error)]
pub enum ParseError {
    /// io error
    #[error("{0}")]
    IoError(#[from] io::Error),
    /// formatting error
    #[error("{0}")]
    FmtError(#[from] fmt::Error),
    /// rphtml specific error
    #[error("{0}")]
    RpHtmlError(#[from] BoxDynError),
    /// the tag is not a valid html
    #[error("Invalid tag: {0}")]
    InvalidTag(String),
}

/// the document is not wrapped with html
pub fn parse_simple<MSG>(html: &str) -> Result<Option<Node<MSG>>, ParseError> {
    let doc = Doc::parse(
        html,
        ParseOptions {
            case_sensitive_tagname: false,
            allow_self_closing: true,
            auto_fix_unclosed_tag: true,
            auto_fix_unexpected_endtag: true,
            auto_fix_unescaped_lt: true,
        },
    )?;
    process_node(doc.get_root_node().borrow().deref())
}

fn process_node<MSG>(node: &rphtml::parser::Node) -> Result<Option<Node<MSG>>, ParseError> {
    let content = if let Some(content) = &node.content {
        let content = String::from_iter(content.iter());
        Some(content)
    } else {
        None
    };

    let mut child_nodes = if let Some(childs) = &node.childs {
        childs
            .iter()
            .flat_map(|child| process_node(child.borrow().deref()).ok().flatten())
            .collect()
    } else {
        vec![]
    };

    match node.node_type {
        NodeType::Tag => {
            let tag = &node.meta.as_ref().expect("must have a tag");
            let tag_name = String::from_iter(tag.borrow().name.iter());
            if let Some(html_tag) = match_tag(&tag_name) {
                let is_self_closing = HTML_SC_TAGS.contains(&html_tag);
                let attributes: Vec<Attribute<MSG>> = tag
                    .borrow()
                    .attrs
                    .iter()
                    .filter_map(|attr| {
                        attr.key
                            .as_ref()
                            .map(|key| {
                                let key = String::from_iter(key.content.iter());
                                if let Some(attr_key) = match_attribute(&key) {
                                    let value = if let Some(value) = &attr.value {
                                        let value = String::from_iter(value.content.iter());
                                        AttributeValue::Simple(Value::from(value))
                                    } else {
                                        AttributeValue::Empty
                                    };
                                    Some(Attribute::new(None, attr_key, value))
                                } else {
                                    log::warn!("Not a standard html attribute: {}", key);
                                    None
                                }
                            })
                            .flatten()
                    })
                    .collect();

                Ok(Some(html_element_self_closing(
                    html_tag,
                    attributes,
                    child_nodes,
                    is_self_closing,
                )))
            } else {
                log::error!("invalid tag: {}", tag_name);
                Err(ParseError::InvalidTag(tag_name))
            }
        }
        NodeType::Text => {
            let content = content.expect("must have a content");
            Ok(Some(text(content)))
        }
        NodeType::AbstractRoot => {
            let child_nodes_len = child_nodes.len();
            match child_nodes_len {
                0 => Ok(None),
                1 => Ok(Some(child_nodes.remove(0))),
                _ => Ok(Some(html_element("html", vec![], child_nodes))),
            }
        }
        _ => Ok(None),
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
        let expected = "<article class=\"side-to-side\"><div>\n        This is div content1\n    </div><footer>\n        This is footer\n    </footer></article>";
        let node: Node<()> = parse_simple(html).ok().flatten().expect("must parse");
        println!("node: {:#?}", node);
        println!("render: {}", node.render_to_string());
        assert_eq!(expected, node.render_to_string());
    }
}
