#![deny(
    warnings,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces
)]
//! a library to parse markdown and convert it into sauron virtual node
#[cfg(feature = "html-parser")]
pub use markdown::html_parser;
pub use markdown::{markdown, markdown_with_plugins, render_markdown, MarkdownParser, Plugins};
/// reexport pulldown cmark
pub use pulldown_cmark;
pub use pulldown_cmark::Tag;
/// reexport sauron
pub use sauron;

mod markdown;
