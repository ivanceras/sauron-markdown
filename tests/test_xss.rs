use sauron::{html::node_list, *};

#[test]
fn anchor() {
    let md = r#"
<a name="n" href="javascript:alert('xss')">*you*</a>
"#;
    println!("md: {}", md);
    let view: Node<()> = node_list(sauron_markdown::parse(md));
    let mut buffer = String::new();
    view.render(&mut buffer).unwrap();
    println!("view: {}", buffer);

    let expected = r#"<p>
    <em>
        <a rel="noopener noreferrer"></a>
        you
    </em>
</p>"#;
    assert_eq!(expected, buffer);
}

#[test]
fn blockqupte_xss() {
    let md = r#"
> hello<a name="n"
> href="javascript:alert('xss')">*you*</a>
"#;
    println!("md: {}", md);
    let view: Node<()> = node_list(sauron_markdown::parse(md));
    let mut buffer = String::new();
    view.render(&mut buffer).unwrap();
    println!("view: {}", buffer);
    let expected = r#"<blockquote>
    <p>
        hello
        <!--separator-->
        href="javascript:alert('xss')"&gt;
        <a rel="noopener noreferrer"></a>
        <em>you</em>
    </p>
</blockquote>"#;

    assert_eq!(expected, buffer);
}
