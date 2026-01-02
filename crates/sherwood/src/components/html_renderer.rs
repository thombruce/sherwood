use super::markdown_parser::MarkdownFile;
use anyhow::Result;
use pulldown_cmark::{Options, Parser, html};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct HtmlRenderer {
    input_dir: PathBuf,
}

impl HtmlRenderer {
    pub fn new(input_dir: &Path) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
        }
    }

    pub fn markdown_to_semantic_html(&self, markdown: &str) -> Result<String> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Ok(self.enhance_semantics(&html_output))
    }

    pub fn generate_blog_list_content(
        &self,
        dir: &Path,
        _list_pages: &HashMap<PathBuf, &MarkdownFile>,
    ) -> Result<String> {
        let mut list_content = String::new();

        // Find all markdown files in this directory (excluding index.md)
        for entry in std::fs::read_dir(self.input_dir.join(dir))? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(extension) = path.extension()
                && (extension == "md" || extension == "markdown")
            {
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                // Skip index files and other list pages
                if !file_name.starts_with("index") {
                    let parsed =
                        super::markdown_parser::MarkdownParser::parse_markdown_file(&path)?;

                    // Generate post list entry using template
                    let date = parsed.frontmatter.date.as_deref();
                    let relative_url_path = path
                        .strip_prefix(&self.input_dir)
                        .unwrap_or(&path)
                        .with_extension("");
                    let relative_url = relative_url_path.to_string_lossy();

                    // Extract first paragraph as excerpt
                    let excerpt = if !self.extract_first_paragraph(&parsed.content).is_empty() {
                        let first_paragraph = self.extract_first_paragraph(&parsed.content);
                        let parser = Parser::new(&first_paragraph);
                        let mut excerpt_html = String::new();
                        html::push_html(&mut excerpt_html, parser);
                        Some(excerpt_html)
                    } else {
                        None
                    };

                    // This would need to be passed in or handled differently
                    // For now, return a simple format
                    let blog_post_html = format!(
                        r#"<article class="blog-post">
    <h3><a href="/{}">{}</a></h3>
    {}{}
</article>"#,
                        relative_url,
                        parsed.title,
                        date.map(|d| format!("<time>{}</time>", d))
                            .unwrap_or_default(),
                        excerpt.map(|e| format!("<p>{}</p>", e)).unwrap_or_default()
                    );

                    list_content.push_str(&blog_post_html);
                    list_content.push_str("\n\n");
                }
            }
        }

        // If no list content was found, return empty string
        if list_content.is_empty() {
            Ok("<!-- No posts found -->".to_string())
        } else {
            Ok(list_content)
        }
    }

    pub fn extract_first_paragraph(&self, content: &str) -> String {
        let mut in_code_block = false;
        let mut lines_since_heading = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip code blocks
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Skip headings and empty lines right after headings
            if trimmed.starts_with('#') {
                lines_since_heading = 0;
                continue;
            }
            if lines_since_heading < 1 {
                lines_since_heading += 1;
                continue;
            }

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Found a paragraph, return it
            return trimmed.to_string();
        }

        String::new()
    }

    fn enhance_semantics(&self, html: &str) -> String {
        let mut enhanced = html.to_string();

        // Wrap paragraphs in semantic sections if they seem like articles
        enhanced = self.wrap_articles(&enhanced);

        // Add semantic structure to lists
        enhanced = self.enhance_lists(&enhanced);

        enhanced
    }

    fn wrap_articles(&self, html: &str) -> String {
        // Simple heuristic: if content has multiple headings, wrap in article tags
        let heading_count = html.matches("<h").count();
        if heading_count > 1 {
            format!("<article>\n{}\n</article>", html)
        } else {
            html.to_string()
        }
    }

    fn enhance_lists(&self, html: &str) -> String {
        // Convert plain lists to more semantic versions when appropriate
        html.replace("<ul>", "<ul class=\"content-list\">")
            .replace("<ol>", "<ol class=\"numbered-list\">")
    }
}
