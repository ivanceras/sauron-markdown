use pulldown_cmark::{Alignment, CodeBlockKind, Event, Options, Parser, Tag};
use sauron::html;
use sauron::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "ammonia")]
use std::{collections::HashSet, iter::FromIterator};

#[cfg(feature = "parse-html")]
pub mod html_parser;

/// convert markdown text to Node
pub fn markdown<MSG>(md: &str) -> Node<MSG> {
    MarkdownParser::from_md(md).node()
}

/// process markdown with plugins
pub fn markdown_with_plugins<MSG>(md: &str, plugins: Plugins<MSG>) -> Node<MSG> {
    MarkdownParser::with_plugins(md, plugins).node()
}

/// parse a markdown string and convert it to Vec<Node>
pub fn render_markdown<MSG>(md: &str) -> Vec<Node<MSG>> {
    MarkdownParser::from_md(md).nodes()
}

/// collections of plugins to be run during the processing of markdown
#[allow(missing_debug_implementations)]
pub struct Plugins<'a, MSG> {
    /// this a function where it is run when a code fence block is detected.
    /// Return an optional new node as a result.
    /// Should return none if the plugin can not process it.
    pub code_fence_processor: Option<Box<dyn Fn(Option<&str>, &str) -> Option<Node<MSG>> + 'a>>,
    /// this is executed for each node in the inline html
    /// Returns a derivative new node if applicable.
    /// Must return None if it the node isn't suitable to be processed.
    pub inline_html_processor: Option<Box<dyn Fn(&Node<MSG>) -> Option<Node<MSG>> + 'a>>,
    /// this is executed for each tag encountered from pulldown-cmark
    pub tag_processor: Option<Box<dyn Fn(&Tag) -> Option<Node<MSG>> + 'a>>,
}

impl<'a, MSG> Default for Plugins<'a, MSG> {
    fn default() -> Self {
        Self {
            code_fence_processor: None,
            inline_html_processor: None,
            tag_processor: None,
        }
    }
}

/// Markdown parser objects, markdown parse state are stored here.
#[allow(missing_debug_implementations)]
pub struct MarkdownParser<'a, MSG> {
    /// contains the top level elements
    elems: Vec<Node<MSG>>,
    /// the elements that are processed
    /// the top of this element is the currently being processed on
    spine: Vec<Node<MSG>>,
    numbers: HashMap<String, usize>,
    /// if h1 is encountered
    is_title_heading: bool,
    /// if a text inside an h1 is encountered
    pub title: Option<String>,
    /// indicates if the text is inside a code block
    in_code_block: bool,
    /// current code fence, ie: it will be `js` if code block is: ```js
    code_fence: Option<String>,
    /// if in a table head , this will convert cell into either th or td
    in_table_head: bool,
    /// a flag if the previous event is inline html or not
    is_prev_inline_html: bool,
    plugins: Plugins<'a, MSG>,
}

impl<'a, MSG> Default for MarkdownParser<'a, MSG> {
    fn default() -> Self {
        MarkdownParser {
            elems: vec![],
            spine: vec![],
            numbers: HashMap::new(),
            is_title_heading: false,
            title: None,
            in_code_block: false,
            code_fence: None,
            in_table_head: false,
            is_prev_inline_html: false,
            plugins: Default::default(),
        }
    }
}

impl<'a, MSG> MarkdownParser<'a, MSG> {
    /// create a markdown parser from a markdown content and the link_lookup replacement
    pub fn from_md(md: &str) -> Self {
        let mut md_parser = Self::default();
        md_parser.do_parse(md);
        md_parser
    }

    /// create a markdown parser from a markdown content with a plugin for custom processing
    pub fn with_plugins(md: &str, plugins: Plugins<'a, MSG>) -> Self {
        let mut md_parser = Self::default();
        md_parser.plugins = plugins;
        md_parser.do_parse(md);
        md_parser
    }

    /// Add a child node to the previous encountered element.
    /// if spine is empty, add it to the top level elements
    fn add_node(&mut self, child: Node<MSG>) {
        if !self.spine.is_empty() {
            let spine_len = self.spine.len();
            self.spine[spine_len - 1]
                .as_element_mut()
                .expect("expecting an element")
                .add_children(vec![child]);
        } else {
            self.elems.push(child);
        }
    }

    /// return the top-level elements
    pub(crate) fn nodes(&self) -> Vec<Node<MSG>> {
        self.elems.clone()
    }

    /// return 1 node, wrapping the the top-level node where there are more than 1.
    pub fn node(&self) -> Node<MSG> {
        if self.elems.len() == 1 {
            self.elems[0].clone()
        } else {
            p(vec![], self.elems.clone())
        }
    }

    fn is_inline_html(ev: &Event) -> bool {
        match ev {
            Event::Html(_) => true,
            _ => false,
        }
    }

    /// start parsing the markdown source
    fn do_parse(&mut self, src: &str) {
        // inline html accumulator
        let mut inline_html = String::new();
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        for ev in Parser::new_ext(src, options) {
            match ev {
                // create a tag and push it to the spine
                Event::Start(ref tag) => {
                    if let Some(ref tag_processor) = self.plugins.tag_processor {
                        if let Some(new_start) = tag_processor(&tag) {
                            self.spine.push(new_start);
                        } else {
                            let start = self.make_tag(&tag);
                            self.spine.push(start);
                        }
                    } else {
                        let start = self.make_tag(&tag);
                        self.spine.push(start);
                    }
                }
                Event::Text(ref content) => {
                    if self.is_title_heading {
                        self.title = Some(content.to_string());
                    }
                    if self.in_code_block {
                        self.add_node(code(
                            if let Some(ref code_fence) = self.code_fence {
                                vec![class(code_fence)]
                            } else {
                                vec![]
                            },
                            if let Some(ref code_fence_processor) =
                                self.plugins.code_fence_processor
                            {
                                let new_node = code_fence_processor(
                                    match self.code_fence {
                                        Some(ref code_fence) => Some(code_fence),
                                        None => None,
                                    },
                                    &content,
                                );
                                if let Some(new_node) = new_node {
                                    vec![new_node]
                                } else {
                                    // the code processor didn't detect it, turn it into a text
                                    // node
                                    vec![text(content)]
                                }
                            } else {
                                // no code fence processor just turn it into a text node
                                vec![text(content)]
                            },
                        ));
                    } else {
                        self.add_node(text(content));
                    }
                }
                Event::SoftBreak => self.add_node(text("\n")),
                Event::HardBreak => self.add_node(br([], [])),
                Event::Code(ref code_str) => self.add_node(code([], [text(code_str)])),
                // ISSUE: html is called for each encountered html tags
                // this needs to be accumulated before it can be parse into actual node
                Event::Html(ref html) => {
                    // accumulate the inline html
                    inline_html += &html;
                }
                Event::FootnoteReference(ref name) => {
                    let len = self.numbers.len() + 1;
                    let number: usize = *self.numbers.entry(name.to_string()).or_insert(len);
                    self.add_node(sup(
                        [class("footnote-reference")],
                        [a([href(format!("#{}", name))], [text(number)])],
                    ));
                }
                Event::Rule => {
                    self.add_node(hr([], []));
                }
                Event::TaskListMarker(ref value) => {
                    self.add_node(input([r#type("checkbox"), checked(*value)], []));
                }
                // end event
                Event::End(ref tag) => self.close_tag(&tag),
            }
            // if inline html is done, process it
            if self.is_prev_inline_html && !Self::is_inline_html(&ev) {
                // not inline html anymore
                self.process_inline_html(&inline_html);
                inline_html.clear();
            }
            self.is_prev_inline_html = Self::is_inline_html(&ev);
        }
        // unprocessed inline html, happens if there is only inline html
        if !inline_html.is_empty() {
            self.process_inline_html(&inline_html);
            inline_html.clear();
        }
    }

    fn make_tag(&mut self, tag: &Tag) -> Node<MSG> {
        match tag {
            Tag::Paragraph => p([], []),
            Tag::Heading(n) => {
                assert!(*n > 0);
                assert!(*n < 7);
                match n {
                    1 => {
                        self.is_title_heading = true;
                        h1([], [])
                    }
                    2 => h2([], []),
                    3 => h3([], []),
                    4 => h4([], []),
                    5 => h5([], []),
                    6 => h6([], []),
                    _ => unreachable!(),
                }
            }
            Tag::BlockQuote => blockquote([], []),
            Tag::CodeBlock(codeblock) => {
                self.in_code_block = true;
                match codeblock {
                    CodeBlockKind::Indented => {
                        self.code_fence = None;
                        code([], [])
                    }
                    CodeBlockKind::Fenced(fence) => {
                        self.code_fence = Some(fence.to_string());
                        code([], [])
                    }
                }
            }
            Tag::List(None) => ul([], []),
            Tag::List(Some(1)) => ol([], []),
            Tag::List(Some(ref start)) => ol([attr("start", *start)], []),
            Tag::Item => li(vec![], vec![]),
            Tag::Table(_alignment) => table([], []),
            Tag::TableHead => {
                self.in_table_head = true;
                tr([], [])
            }
            Tag::TableRow => {
                self.in_table_head = false;
                tr([], [])
            }
            Tag::TableCell => {
                if self.in_table_head {
                    th([], [])
                } else {
                    td([], [])
                }
            }
            Tag::Emphasis => html::em([], []),
            Tag::Strong => strong([], []),
            Tag::Strikethrough => s([], []),
            Tag::Link(_link_type, ref link_href, ref link_title) => a(
                [
                    href(link_href.to_string()),
                    html::attributes::title(link_title.to_string()),
                ],
                [],
            ),
            Tag::Image(_link_type, ref image_src, ref image_title) => img(
                [
                    src(image_src.to_string()),
                    html::attributes::title(image_title.to_string()),
                ],
                [],
            ),
            Tag::FootnoteDefinition(name) => {
                let len = self.numbers.len() + 1;
                let number = self.numbers.entry(name.to_string()).or_insert(len);
                footer(
                    [class("footnote-definition"), id(name.to_string())],
                    [sup([class("footnote-label")], [text(number)])],
                )
            }
        }
    }

    fn close_tag(&mut self, tag: &Tag) {
        let spine_len = self.spine.len();
        assert!(spine_len >= 1);
        let mut top = self.spine.pop().expect("must have one element");

        match tag {
            Tag::Heading(1) => self.is_title_heading = false,
            Tag::CodeBlock(_) => self.in_code_block = false,
            Tag::Table(aligns) => {
                if let Some(element) = top.as_element_mut() {
                    for r in element.children_mut() {
                        if let Some(r) = r.as_element_mut() {
                            for (i, c) in r.children_mut().iter_mut().enumerate() {
                                if let Some(tag) = c.as_element_mut() {
                                    match aligns[i] {
                                        Alignment::None => {}
                                        Alignment::Left => {
                                            tag.add_attributes(vec![class("text-left")])
                                        }
                                        Alignment::Center => {
                                            tag.add_attributes(vec![class("text-center")])
                                        }
                                        Alignment::Right => {
                                            tag.add_attributes(vec![class("text-right")])
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => (),
        }
        self.add_node(top);
    }

    fn process_inline_html(&mut self, inline_html: &str) {
        #[cfg(feature = "parse-html")]
        if let Ok(nodes) = html_parser::parse_simple(&inline_html) {
            for node in nodes {
                let new_node = self.run_inline_processor(node);
                self.add_node(new_node);
            }
        }
        // if no html-parser is not included,
        // then just escape the html
        #[cfg(not(feature = "parse-html"))]
        {
            let escaped_text = html_escape::encode_text(inline_html);
            self.add_node(text(escaped_text));
        }
    }

    /// Run a plugin processor to elements in inline html
    /// if it the plugin produces a Node it will be return as is.
    /// If the plugin doesn't produce a node, return the current node
    #[allow(unused)]
    fn run_inline_processor(&self, mut node: Node<MSG>) -> Node<MSG> {
        if let Some(ref inline_html_processor) = self.plugins.inline_html_processor {
            let new_node = inline_html_processor(&node);
            if let Some(new_node) = new_node {
                return new_node;
            } else {
                if let Some(element) = node.as_element_mut() {
                    let mut new_children = vec![];
                    for child in element.children.drain(..) {
                        let new_child = self.run_inline_processor(child);
                        new_children.push(new_child)
                    }
                    node.add_children_ref_mut(new_children);
                }
                node
            }
        } else {
            node
        }
    }
}
