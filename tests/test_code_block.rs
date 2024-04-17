use sauron::{html::node_list, *};
use sauron_markdown::parse;

#[test]
fn test_md_with_html_inline_processor() {
    let md = r#"```rust
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
</pre>"#;
    let node: Node<()> = node_list(parse(md));

    let html = node.render_to_string();
    let expected = "\
<code class=\"rust\">\
fn main(){\
\n    println!(\"this is real code block here\");\
\n}\
\n</code><pre></pre><code></code>        struct Foo {\
\n            int bar;\
\n            date baz;\
\n            string quux;\
\n          };\
\n          //somewhere in something\
\n          Array    \n";
    assert_eq!(html, expected);
}
