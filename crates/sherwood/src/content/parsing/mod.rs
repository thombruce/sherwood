pub mod ast_utils;
pub mod excerpt_extraction;
pub mod frontmatter_parsing;
pub mod html_conversion;
pub mod markdown_parser;
pub mod title_extraction;

// Re-exports for backward compatibility and convenience
pub use frontmatter_parsing::Frontmatter;
pub use markdown_parser::{MarkdownFile, MarkdownParser};

// Re-export utility functions for advanced usage
pub use ast_utils::extract_text_from_nodes;
pub use excerpt_extraction::ExcerptExtractor;
pub use frontmatter_parsing::FrontmatterParser;
pub use html_conversion::HtmlConverter;
pub use title_extraction::{extract_title_from_ast, extract_title_from_path, resolve_title};
