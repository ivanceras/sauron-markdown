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
pub use markdown::{parse, parse_with_title};
/// reexport pulldown cmark
pub use pulldown_cmark;
pub use pulldown_cmark::Tag;
/// reexport sauron
pub use sauron;

mod markdown;
