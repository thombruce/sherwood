//! Turning content files into [`page::Page`]s.
//!
//! [`frontmatter`] splits the metadata block from the body, [`parser`] holds the
//! pluggable [`parser::ContentParser`] system (markdown built in), and
//! [`page::load_page`] ties them together with path-derived fields.

pub mod frontmatter;
pub mod page;
pub mod parser;
