use sauron::{html::node_list, *};
use sauron_markdown::parse;

#[test]
fn test_md_with_html_inline_processor() {
    let md = r#"
```rust
fn main(){
    println!("this is real code block here");
}
```
<pre>
    <code>
        struct Foo {
            int bar;
            date baz;
            string quux;
          };

          //somewhere in something
          Array&lt;Foo&gt; foos;
    </code>
</pre>
        "#;
    let node: Node<()> = node_list(parse(md));

    let html = node.render_to_string();
    println!("html: {}", html);
    dbg!(&html);
    let expected = r#"<p><pre><code class="rust">fn main(){
    println!("this is real code block here");
}
</code></pre><pre><code class="highlighted">
        struct Foo {
            int bar;
            date baz;
            string quux;
          };

          //somewhere in something
          Array<Foo> foos;
    <p>--highlighted already</p></code></pre></p>"#;
    assert_eq!(expected, html);
}
