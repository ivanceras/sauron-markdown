use sauron_markdown::sauron::{html::node_list, *};
use sauron_markdown::*;

#[test]
fn test_inline_htmls() {
    let md = r#"<article class="side-to-side">
    <div>
        This is div content1
    </div>
    <footer>
        This is footer
    </footer>
</article>"#;

    let view: Node<()> = node_list(parse(md));

    let mut buffer = String::new();
    view.render(&mut buffer).unwrap();
    println!("view: {}", buffer);
    assert_eq!(md, buffer);
}

#[test]
fn source_code() {
    let md = r#"
```rust
    fn main(){
        println!("Hello world!");
    }
```
        "#;
    let expected = "<pre>\n    <code class=\"rust\">    fn main(){\n        println!(\"Hello world!\");\n    }\n</code>\n</pre>";
    let view: Node<()> = node_list(parse(md));

    let mut buffer = String::new();
    view.render(&mut buffer).unwrap();
    println!("view: {}", buffer);
    assert_eq!(expected, buffer);
}

#[test]
fn code() {
    let md = r#"
This is has some `code` and other..
        "#;
    let expected = "<p>\n    This is has some \n    <code>code</code>\n     and other..\n</p>";
    let view: Node<()> = node_list(parse(md));

    let mut buffer = String::new();
    view.render(&mut buffer).unwrap();
    println!("view: {}", buffer);
    assert_eq!(expected, buffer);
}

#[test]
fn footnotes() {
    let md = r#"
### [Footnotes](https://github.com/markdown-it/markdown-it-footnote)

Footnote 1 link[^first].

Footnote 2 link[^second].

Inline footnote^[Text of inline footnote] definition.

Duplicated footnote reference[^second].

[^first]: Footnote **can have markup**

    and multiple paragraphs.

[^second]: Footnote text.
        "#;

    let expected = "<p><h3><a href=\"https://github.com/markdown-it/markdown-it-footnote\" title=\"\">Footnotes</a></h3><p>Footnote 1 link<sup class=\"footnote-reference\"><a href=\"#first\">1</a></sup>.</p><p>Footnote 2 link<sup class=\"footnote-reference\"><a href=\"#second\">2</a></sup>.</p><p>Inline footnote^<!--separator-->[<!--separator-->Text of inline footnote<!--separator-->]<!--separator--> definition.</p><p>Duplicated footnote reference<sup class=\"footnote-reference\"><a href=\"#second\">2</a></sup>.</p><footer class=\"footnote-definition\" id=\"first\"><sup class=\"footnote-label\">1</sup><p>Footnote <strong>can have markup</strong></p></footer><pre><code>and multiple paragraphs.\n</code></pre><footer class=\"footnote-definition\" id=\"second\"><sup class=\"footnote-label\">2</sup><p>Footnote text.</p></footer></p>";
    let view: Node<()> = node_list(parse(md));

    assert_eq!(expected, view.render_to_string());
}

#[test]
fn test_md_with_html() {
    let md = r#"
[Hello](link.html)
<img src="img.jpeg"/>"#;

    let expected =
"<p>\n    <p>\n        <a href=\"link.html\" title=\"\">Hello</a>\n        \n\n    </p>\n    <img src=\"img.jpeg\"/>\n</p>";
    let view: Node<()> = node_list(parse(md));

    let mut buffer = String::new();
    view.render(&mut buffer).unwrap();
    println!("view: {}", buffer);
    assert_eq!(expected, buffer);
}

#[test]
fn test_md_with_image() {
    let md = r#"
[Hello](link.html)
![](img.jpeg "Image title")"#;

    let expected =
            "<p>\n    <a href=\"link.html\" title=\"\">Hello</a>\n    \n\n    <img src=\"img.jpeg\" title=\"Image title\"/>\n</p>";
    let view: Node<()> = node_list(parse(md));

    let mut buffer = String::new();
    view.render(&mut buffer).unwrap();
    println!("view: {}", buffer);
    assert_eq!(expected, buffer);
}

#[test]
fn test_list() {
    let md = r#"
# List
- list 1
- list 2
- list 3
    - sublist 1
        - some other sublist A
        - some other sublist B
    - sublist 2
    - sublist 3
"#;
    let expected = r#"<p><h1>List</h1><ul><li>list 1</li><li>list 2</li><li>list 3<ul><li>sublist 1<ul><li>some other sublist A</li><li>some other sublist B</li></ul></li><li>sublist 2</li><li>sublist 3</li></ul></li></ul></p>"#;
    let view: Node<()> = node_list(parse(md));
    let mut buffer = String::new();
    view.render_compressed(&mut buffer).unwrap();
    println!("view: {}", buffer);
    assert_eq!(expected, buffer);
}

#[test]
fn test_md() {
    let md = r#"
An h1 header
============
look like:
  * this one
  * that one
  * the other one"#;
    let view: Node<()> = node_list(parse(md));

    let expected = r#"<p>
    <h1>An h1 header</h1>
    <p>look like:</p>
    <ul>
        <li>this one</li>
        <li>that one</li>
        <li>the other one</li>
    </ul>
</p>"#;

    let mut buffer = String::new();
    view.render(&mut buffer).unwrap();
    println!("view: {}", buffer);
    assert_eq!(expected, buffer);
}

#[test]
fn test_md_links() {
    let md = r#"
[link text](http://dev.nodeca.com)

[link with title](http://nodeca.github.io/pica/demo/ "title text!")"#;
    let view: Node<()> = node_list(parse(md));
    let expected = r#"<p>
    <p>
        <a href="http://dev.nodeca.com" title="">link text</a>
    </p>
    <p>
        <a href="http://nodeca.github.io/pica/demo/" title="title text!">link with title</a>
    </p>
</p>"#;

    let mut buffer = String::new();
    view.render(&mut buffer).unwrap();
    println!("view: {}", buffer);
    assert_eq!(expected, buffer);
}

#[test]
fn test_md_tables() {
    let md = r#"
## Tables

| Option | Description |
|:------ | -----------:|
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |
}
"#;
    let view: Node<()> = node_list(parse(md));
    let expected = r#"<p>
    <h2>Tables</h2>
    <table>
        <tr>
            <th class="text-left">Option</th>
            <th class="text-right">Description</th>
        </tr>
        <tr>
            <td class="text-left">data</td>
            <td class="text-right">path to data files to supply the data that will be passed into templates.</td>
        </tr>
        <tr>
            <td class="text-left">engine</td>
            <td class="text-right">engine to be used for processing templates. Handlebars is the default.</td>
        </tr>
        <tr>
            <td class="text-left">ext</td>
            <td class="text-right">extension to be used for dest files.</td>
        </tr>
        <tr>
            <td class="text-left">}</td>
            <td class="text-right"></td>
        </tr>
    </table>
</p>"#;

    let mut buffer = String::new();
    view.render(&mut buffer).unwrap();
    println!("view: {}", buffer);
    assert_eq!(expected, buffer);
}

#[test]
fn test_md_with_svgbob_processor() {
    let md = r#"
    This is <b>Markdown</b> with some <i>funky</i> __examples__.
```bob
      .------.       +-------+
      | bob  | *---> | alice |
      `------'       +-------+
```
            "#;
    let node: Node<()> = node_list(parse(md));

    let html = node.render_to_string();
    println!("html: {}", html);
    dbg!(&html);
    let expected = r#"<p><pre><code>This is <b>Markdown</b> with some <i>funky</i> __examples__.
</code></pre><pre><code class="bob"><svg xmlns="http://www.w3.org/2000/svg" width="248" height="64"><style>line, path, circle,rect,polygon{stroke:black;stroke-width:2;stroke-opacity:1;fill-opacity:1;stroke-linecap:round;stroke-linejoin:miter;}text{font-family:monospace;font-size:14px;}rect.backdrop{stroke:none;fill:white;}.broken{stroke-dasharray:8;}.filled{fill:black;}.bg_filled{fill:white;}.nofill{fill:white;}.end_marked_arrow{marker-end:url(#arrow);}.start_marked_arrow{marker-start:url(#arrow);}.end_marked_diamond{marker-end:url(#diamond);}.start_marked_diamond{marker-start:url(#diamond);}.end_marked_circle{marker-end:url(#circle);}.start_marked_circle{marker-start:url(#circle);}.end_marked_open_circle{marker-end:url(#open_circle);}.start_marked_open_circle{marker-start:url(#open_circle);}.end_marked_big_open_circle{marker-end:url(#big_open_circle);}.start_marked_big_open_circle{marker-start:url(#big_open_circle);}</style><defs><marker id="arrow" viewBox="-2 -2 8 8" refX="4" refY="2" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><polygon points="0,0 0,4 4,2 0,0"></polygon></marker><marker id="diamond" viewBox="-2 -2 8 8" refX="4" refY="2" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><polygon points="0,2 2,0 4,2 2,4 0,2"></polygon></marker><marker id="circle" viewBox="0 0 8 8" refX="4" refY="4" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><circle cx="4" cy="4" r="2" class="filled"></circle></marker><marker id="open_circle" viewBox="0 0 8 8" refX="4" refY="4" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><circle cx="4" cy="4" r="2" class="bg_filled"></circle></marker><marker id="big_open_circle" viewBox="0 0 8 8" refX="4" refY="4" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><circle cx="4" cy="4" r="3" class="bg_filled"></circle></marker></defs><rect class="backdrop" x="0" y="0" width="248" height="64"></rect><rect x="52" y="8" width="56" height="32" class="solid nofill" rx="4"></rect><text x="66" y="28" >bob</text><rect x="172" y="8" width="64" height="32" class="solid nofill" rx="0"></rect><text x="186" y="28" >alice</text><circle cx="124" cy="24" r="3" class="filled"></circle><g><line x1="128" y1="24" x2="152" y2="24" class="solid"></line><polygon points="152,20 160,24 152,28" class="filled"></polygon></g></svg></code></pre></p>"#;
    assert_eq!(expected, html);
}
