use crate::sauron::{html, html::attributes, *};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

pub(crate) struct MdParser<MSG> {
    spine: Vec<Node<MSG>>,
    nodes: Vec<Node<MSG>>,
}
impl<MSG> MdParser<MSG> {
    fn new() -> Self {
        Self {
            spine: vec![],
            nodes: vec![],
        }
    }
    fn add_child(&mut self, child: Node<MSG>) {
        log::info!("adding child: {:?}", child);
        assert!(!self.spine.is_empty());
        if let Some(last) = self.spine.last_mut() {
            last.add_children([child]).expect("add children");
        }
    }

    /// push node to the spine
    fn push_to_spine(&mut self, node: Node<MSG>) {
        self.spine.push(node);
    }

    /// push as a new node
    fn push_to_nodes(&mut self, node: Node<MSG>) {
        self.nodes.push(node);
    }

    fn parse(mut self, src: &str) -> Vec<Node<MSG>> {
        for ev in Parser::new_ext(src, Options::all()) {
            match ev {
                Event::Start(tag) => {
                    self.push_to_spine(make_tag(&tag));
                }
                Event::End(_tag) => {
                    assert!(!self.spine.is_empty());
                    //TODO: there is a big code smell here
                    let len = self.spine.len();
                    let top = self.spine.pop().unwrap();
                    if len == 1 {
                        self.push_to_nodes(top);
                    } else {
                        // TODO: code smell
                        // we are adding the top children if there are more than 2 elemen in the
                        // spine
                        self.spine[len - 2]
                            .add_children([top])
                            .expect("add children");
                    }
                }
                Event::Text(text_) => self.add_child(text(text_)),
                Event::SoftBreak => self.add_child(text("\n")),
                Event::HardBreak => self.add_child(br([], [])),
                Event::Html(html) => {
                    if self.spine.is_empty() {
                        self.push_to_nodes(safe_html(html));
                    } else {
                        self.add_child(safe_html(html))
                    }
                }
                Event::Code(content) => self.add_child(text(content)),
                Event::Rule => {
                    // <hr> rule is top level element
                    self.push_to_nodes(hr([], []));
                }
                Event::FootnoteReference(name) => self.add_child(a([href(name.to_string())], [])),
                Event::TaskListMarker(ref value) => {
                    self.add_child(input([r#type("checkbox"), checked(*value)], []));
                }
            }
        }
        self.nodes
    }
}

fn make_tag<MSG>(t: &Tag) -> Node<MSG> {
    match t {
        Tag::Paragraph => p([], []),
        Tag::Heading(n) => {
            assert!(*n > 0);
            assert!(*n < 7);
            match n {
                1 => h1([], []),
                2 => h2([], []),
                3 => h3([], []),
                4 => h4([], []),
                5 => h5([], []),
                6 => h6([], []),
                _ => unreachable!(),
            }
        }
        Tag::BlockQuote => blockquote([], []),
        Tag::CodeBlock(kind) => {
            let attributes = if let CodeBlockKind::Fenced(lang) = kind {
                class(lang.to_string())
            } else {
                empty_attr()
            };
            code([attributes], [])
        }
        Tag::List(None) => ul([], []),
        Tag::List(Some(1)) => ol([], []),
        Tag::List(Some(start)) => ol([attr("start", start.to_string())], []),
        Tag::Item => li([], []),
        Tag::Table(_alignment) => table([], []),
        Tag::TableHead => th([], []),
        Tag::TableRow => tr([], []),
        Tag::TableCell => td([], []),
        Tag::Emphasis => em([], []),
        Tag::Strong => strong([], []),
        Tag::Link(_type, href_, title_) => a(
            [
                href(href_.to_string()),
                attributes::title(title_.to_string()),
            ],
            [],
        ),
        Tag::Image(_type, src_, title_) => img(
            [src(src_.to_string()), attributes::title(title_.to_string())],
            [],
        ),
        Tag::Strikethrough => html::s([], []),
        Tag::FootnoteDefinition(footnote_id) => footer(
            [class("footnote-definition"), id(footnote_id.to_string())],
            [],
        ),
    }
}

/// parse the markdown and return the nodes
pub fn parse<MSG>(src: &str) -> Vec<Node<MSG>> {
    MdParser::new().parse(src)
}

fn maybe_title<MSG>(node: &Node<MSG>) -> Option<&str> {
    if node.tag() == Some(&&"h1") {
        println!("# an h1 here..");
        println!("{}", node.render_to_string());
        let children = node.children();
        if children.len() == 1 {
            return children[0].as_text();
        }
    }
    None
}

fn find_title<MSG>(nodes: &[Node<MSG>]) -> Option<&str> {
    for node in nodes.iter() {
        if let Some(title) = maybe_title(node) {
            return Some(title);
        } else {
            if let Some(title) = find_title(node.children()) {
                return Some(title);
            }
        }
    }
    None
}

/// parse the markdown and return the first encountered h1 text and the nodes
pub fn parse_with_title<MSG>(src: &str) -> (Option<String>, Vec<Node<MSG>>) {
    let nodes = parse(src);
    let title = find_title(&nodes).map(|t| t.to_string());
    (title, nodes)
}
