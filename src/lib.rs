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
pub use markdown::{markdown, markdown_with_plugins, render_markdown, MarkdownParser, Plugins};
pub use pulldown_cmark::Tag;
mod markdown;
