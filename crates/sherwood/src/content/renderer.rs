use super::parser::MarkdownFile;
use crate::presentation::templates::TemplateManager;
use anyhow::Result;
use chrono::NaiveDate;
use markdown::{Options, to_html_with_options};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct SortConfig {
    field: String,
    order: String,
}

impl SortConfig {
    fn from_frontmatter(frontmatter: &super::parser::Frontmatter) -> Self {
        let field = frontmatter
            .sort_by
            .as_ref()
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "date".to_string());

        let order = frontmatter
            .sort_order
            .as_ref()
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| {
                if field == "date" {
                    "desc".to_string()
                } else {
                    "asc".to_string()
                }
            });

        Self { field, order }
    }

    fn is_valid_field(field: &str) -> bool {
        matches!(field, "date" | "title" | "filename")
    }

    fn is_valid_order(order: &str) -> bool {
        matches!(order, "asc" | "desc")
    }
}

pub struct HtmlRenderer {
    input_dir: PathBuf,
    template_manager: TemplateManager,
}

impl HtmlRenderer {
    pub fn new(input_dir: &Path, template_manager: TemplateManager) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            template_manager,
        }
    }

    fn sort_markdown_files(&self, files: &mut [MarkdownFile], sort_config: &SortConfig) {
        // Validate sort configuration
        let field = if SortConfig::is_valid_field(&sort_config.field) {
            &sort_config.field
        } else {
            eprintln!(
                "Warning: Invalid sort field '{}', falling back to 'date'",
                sort_config.field
            );
            "date"
        };

        let order = if SortConfig::is_valid_order(&sort_config.order) {
            &sort_config.order
        } else {
            eprintln!(
                "Warning: Invalid sort order '{}', falling back to 'asc'",
                sort_config.order
            );
            "asc"
        };

        files.sort_by(|a, b| {
            let comparison = match field {
                "date" => self.compare_by_date(a, b),
                "title" => a.title.cmp(&b.title),
                "filename" => self.compare_by_filename(a, b),
                _ => Ordering::Equal, // Should not reach here due to validation
            };

            if order == "desc" {
                comparison.reverse()
            } else {
                comparison
            }
        });
    }

    fn compare_by_date(&self, a: &MarkdownFile, b: &MarkdownFile) -> Ordering {
        match (&a.frontmatter.date, &b.frontmatter.date) {
            (Some(date_a), Some(date_b)) => {
                match (self.parse_date(date_a), self.parse_date(date_b)) {
                    (Some(parsed_a), Some(parsed_b)) => parsed_a.cmp(&parsed_b),
                    (Some(_), None) => Ordering::Less, // Valid date comes before invalid
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => self.compare_by_filename(a, b), // Both invalid, fall back to filename
                }
            }
            (Some(_), None) => Ordering::Less, // File with date comes before file without
            (None, Some(_)) => Ordering::Greater,
            (None, None) => self.compare_by_filename(a, b), // Neither has date, fall back to filename
        }
    }

    fn compare_by_filename(&self, a: &MarkdownFile, b: &MarkdownFile) -> Ordering {
        a.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .cmp(b.path.file_name().and_then(|n| n.to_str()).unwrap_or(""))
    }

    fn parse_date(&self, date_str: &str) -> Option<NaiveDate> {
        // Try ISO format first (YYYY-MM-DD)
        if let Ok(date) = NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d") {
            return Some(date);
        }

        // Try other common formats
        let formats = [
            "%B %d, %Y", // "January 15, 2024"
            "%b %d, %Y", // "Jan 15, 2024"
            "%d/%m/%Y",  // "15/01/2024"
            "%m/%d/%Y",  // "01/15/2024"
            "%Y-%m-%d",  // "2024-01-15" (duplicate but ensures we try again)
        ];

        for format in &formats {
            if let Ok(date) = NaiveDate::parse_from_str(date_str.trim(), format) {
                return Some(date);
            }
        }

        None
    }

    /// Process content intelligently - HTML passes through, markdown gets converted
    pub fn process_content(&self, content: &str) -> Result<String> {
        // Simple HTML detection - if it looks like HTML, treat as HTML
        if self.looks_like_html(content) {
            self.validate_basic_html(content)?;
            Ok(content.to_string())
        } else {
            // Fallback to markdown processing for backward compatibility
            self.markdown_to_semantic_html(content)
        }
    }

    pub fn markdown_to_semantic_html(&self, markdown: &str) -> Result<String> {
        let options = Options::gfm(); // GFM includes strikethrough, tables, footnotes

        let html_output = to_html_with_options(markdown, &options)
            .map_err(|e| anyhow::anyhow!("Failed to parse markdown: {}", e))?;

        Ok(self.enhance_semantics(&html_output))
    }

    /// Basic heuristic to detect if content is already HTML
    fn looks_like_html(&self, content: &str) -> bool {
        let trimmed = content.trim_start();
        trimmed.starts_with('<') && (trimmed.contains("</") || trimmed.contains("/>"))
    }

    /// Basic HTML validation - check for balanced tags and safe elements
    fn validate_basic_html(&self, html: &str) -> Result<()> {
        // Simple validation: check for dangerous elements
        let dangerous = ["<script", "<iframe", "<object", "<embed", "<form"];
        let lower_html = html.to_lowercase();

        for danger in &dangerous {
            if lower_html.contains(danger) {
                return Err(anyhow::anyhow!(
                    "HTML contains potentially unsafe element: {}",
                    danger
                ));
            }
        }

        Ok(())
    }

    // TODO: Implement direct AST-to-HTML rendering when markdown-rs supports it
    // Future enhancement: Use AST directly for HTML generation instead of:
    // 1. AST parsing → content extraction → markdown string → HTML parsing
    // This would eliminate the double parsing and provide more efficient rendering
    // Track progress: https://github.com/wooorm/markdown-rs/issues

    pub fn generate_blog_list_content(
        &self,
        dir: &Path,
        list_pages: &HashMap<PathBuf, &MarkdownFile>,
    ) -> Result<String> {
        // Find the list page for this directory to get sort configuration
        let sort_config = list_pages
            .get(dir)
            .map(|list_page| SortConfig::from_frontmatter(&list_page.frontmatter))
            .unwrap_or_else(|| SortConfig {
                field: "date".to_string(),
                order: "desc".to_string(),
            });

        let mut markdown_files = Vec::new();

        // Collect all markdown files in this directory (excluding index.md)
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
                    let parsed = super::parser::MarkdownParser::parse_markdown_file(&path)?;
                    markdown_files.push(parsed);
                }
            }
        }

        // Sort the collected files
        self.sort_markdown_files(&mut markdown_files, &sort_config);

        // Generate HTML content
        let mut list_content = String::new();
        for parsed in markdown_files {
            // Generate post list entry using template
            let date = parsed.frontmatter.date.as_deref();
            let relative_url_path = parsed
                .path
                .strip_prefix(&self.input_dir)
                .unwrap_or(&parsed.path)
                .with_extension("");
            let relative_url = relative_url_path.to_string_lossy();

            // Extract first paragraph as excerpt
            let excerpt = if !self.extract_first_paragraph(&parsed.content).is_empty() {
                let first_paragraph = self.extract_first_paragraph(&parsed.content);
                let excerpt_html = self.process_content(&first_paragraph)?;
                Some(excerpt_html)
            } else {
                None
            };

            // Use the template to render each content item
            let content_item_html = self.template_manager.render_content_item(
                &parsed.title,
                &relative_url,
                date,
                excerpt.as_deref(),
            )?;

            list_content.push_str(&content_item_html);
            list_content.push_str("\n\n");
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
        enhanced = wrap_articles(&enhanced);

        // Add semantic structure to lists
        enhanced = enhance_lists(&enhanced);

        enhanced
    }
}

fn wrap_articles(html: &str) -> String {
    // Simple heuristic: if content has multiple headings, wrap in article tags
    let heading_count = html.matches("<h").count();
    if heading_count > 1 {
        format!("<article>\n{}\n</article>", html)
    } else {
        html.to_string()
    }
}

fn enhance_lists(html: &str) -> String {
    // Convert plain lists to more semantic versions when appropriate
    html.replace("<ul>", "<ul class=\"content-list\">")
        .replace("<ol>", "<ol class=\"numbered-list\">")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::parser::{Frontmatter, MarkdownParser};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_markdown_file(
        temp_dir: &tempfile::TempDir,
        filename: &str,
        frontmatter: &str,
        content: &str,
    ) -> std::path::PathBuf {
        let file_path = temp_dir.path().join(filename);
        let full_content = format!("{}\n\n{}", frontmatter, content);
        fs::write(&file_path, full_content).unwrap();
        file_path
    }

    fn create_test_html_renderer() -> HtmlRenderer {
        use crate::presentation::templates::TemplateManager;
        let temp_dir = tempdir().unwrap();
        let template_manager = TemplateManager::new(temp_dir.path()).unwrap();
        HtmlRenderer::new(temp_dir.path(), template_manager)
    }

    #[test]
    fn test_sort_config_from_frontmatter() {
        let frontmatter = Frontmatter {
            sort_by: Some("title".to_string()),
            sort_order: Some("desc".to_string()),
            ..Default::default()
        };

        let config = SortConfig::from_frontmatter(&frontmatter);
        assert_eq!(config.field, "title");
        assert_eq!(config.order, "desc");
    }

    #[test]
    fn test_sort_config_defaults() {
        let frontmatter = Frontmatter::default();

        let config = SortConfig::from_frontmatter(&frontmatter);
        assert_eq!(config.field, "date");
        assert_eq!(config.order, "desc");
    }

    #[test]
    fn test_sort_config_field_validation() {
        assert!(SortConfig::is_valid_field("date"));
        assert!(SortConfig::is_valid_field("title"));
        assert!(SortConfig::is_valid_field("filename"));
        assert!(!SortConfig::is_valid_field("invalid"));
        assert!(!SortConfig::is_valid_field("author"));
    }

    #[test]
    fn test_sort_config_order_validation() {
        assert!(SortConfig::is_valid_order("asc"));
        assert!(SortConfig::is_valid_order("desc"));
        assert!(!SortConfig::is_valid_order("ascending"));
        assert!(!SortConfig::is_valid_order("invalid"));
    }

    #[test]
    fn test_date_parsing() {
        let renderer = create_test_html_renderer();

        // Test ISO format
        assert!(renderer.parse_date("2024-01-15").is_some());

        // Test other formats
        assert!(renderer.parse_date("January 15, 2024").is_some());
        assert!(renderer.parse_date("Jan 15, 2024").is_some());
        assert!(renderer.parse_date("15/01/2024").is_some());
        assert!(renderer.parse_date("01/15/2024").is_some());

        // Test invalid format
        assert!(renderer.parse_date("invalid date").is_none());
    }

    #[test]
    fn test_sort_by_date_ascending() {
        let renderer = create_test_html_renderer();

        let file1 = MarkdownFile {
            path: PathBuf::from("file1.md"),
            content: "Content 1".to_string(),
            title: "File 1".to_string(),
            frontmatter: Frontmatter {
                date: Some("2024-01-10".to_string()),
                ..Default::default()
            },
        };

        let file2 = MarkdownFile {
            path: PathBuf::from("file2.md"),
            content: "Content 2".to_string(),
            title: "File 2".to_string(),
            frontmatter: Frontmatter {
                date: Some("2024-01-15".to_string()),
                ..Default::default()
            },
        };

        let mut files = vec![file2, file1];
        let config = SortConfig {
            field: "date".to_string(),
            order: "asc".to_string(),
        };

        renderer.sort_markdown_files(&mut files, &config);

        assert_eq!(files[0].frontmatter.date, Some("2024-01-10".to_string()));
        assert_eq!(files[1].frontmatter.date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_sort_by_date_descending() {
        let renderer = create_test_html_renderer();

        let file1 = MarkdownFile {
            path: PathBuf::from("file1.md"),
            content: "Content 1".to_string(),
            title: "File 1".to_string(),
            frontmatter: Frontmatter {
                date: Some("2024-01-10".to_string()),
                ..Default::default()
            },
        };

        let file2 = MarkdownFile {
            path: PathBuf::from("file2.md"),
            content: "Content 2".to_string(),
            title: "File 2".to_string(),
            frontmatter: Frontmatter {
                date: Some("2024-01-15".to_string()),
                ..Default::default()
            },
        };

        let mut files = vec![file1.clone(), file2.clone()];
        let config = SortConfig {
            field: "date".to_string(),
            order: "desc".to_string(),
        };

        renderer.sort_markdown_files(&mut files, &config);

        assert_eq!(files[0].frontmatter.date, Some("2024-01-15".to_string()));
        assert_eq!(files[1].frontmatter.date, Some("2024-01-10".to_string()));
    }

    #[test]
    fn test_sort_by_title() {
        let renderer = create_test_html_renderer();

        let file1 = MarkdownFile {
            path: PathBuf::from("z_file.md"),
            content: "Content 1".to_string(),
            title: "Zebra".to_string(),
            frontmatter: Frontmatter::default(),
        };

        let file2 = MarkdownFile {
            path: PathBuf::from("a_file.md"),
            content: "Content 2".to_string(),
            title: "Apple".to_string(),
            frontmatter: Frontmatter::default(),
        };

        let mut files = vec![file1, file2];
        let config = SortConfig {
            field: "title".to_string(),
            order: "asc".to_string(),
        };

        renderer.sort_markdown_files(&mut files, &config);

        assert_eq!(files[0].title, "Apple");
        assert_eq!(files[1].title, "Zebra");
    }

    #[test]
    fn test_sort_by_filename() {
        let renderer = create_test_html_renderer();

        let file1 = MarkdownFile {
            path: PathBuf::from("z_file.md"),
            content: "Content 1".to_string(),
            title: "Zebra".to_string(),
            frontmatter: Frontmatter::default(),
        };

        let file2 = MarkdownFile {
            path: PathBuf::from("a_file.md"),
            content: "Content 2".to_string(),
            title: "Apple".to_string(),
            frontmatter: Frontmatter::default(),
        };

        let mut files = vec![file1, file2];
        let config = SortConfig {
            field: "filename".to_string(),
            order: "asc".to_string(),
        };

        renderer.sort_markdown_files(&mut files, &config);

        assert_eq!(
            files[0].path.file_name().unwrap().to_str().unwrap(),
            "a_file.md"
        );
        assert_eq!(
            files[1].path.file_name().unwrap().to_str().unwrap(),
            "z_file.md"
        );
    }

    #[test]
    fn test_sort_with_missing_dates() {
        let renderer = create_test_html_renderer();

        let file_with_date = MarkdownFile {
            path: PathBuf::from("with_date.md"),
            content: "Content 1".to_string(),
            title: "With Date".to_string(),
            frontmatter: Frontmatter {
                date: Some("2024-01-15".to_string()),
                ..Default::default()
            },
        };

        let file_without_date = MarkdownFile {
            path: PathBuf::from("without_date.md"),
            content: "Content 2".to_string(),
            title: "Without Date".to_string(),
            frontmatter: Frontmatter::default(),
        };

        let mut files = vec![file_without_date, file_with_date];
        let config = SortConfig {
            field: "date".to_string(),
            order: "asc".to_string(),
        };

        renderer.sort_markdown_files(&mut files, &config);

        // Files with dates should come before files without dates
        assert_eq!(files[0].frontmatter.date, Some("2024-01-15".to_string()));
        assert_eq!(files[1].frontmatter.date, None);
    }

    #[test]
    fn test_sort_with_invalid_dates() {
        let renderer = create_test_html_renderer();

        let file_with_valid_date = MarkdownFile {
            path: PathBuf::from("valid_date.md"),
            content: "Content 1".to_string(),
            title: "Valid Date".to_string(),
            frontmatter: Frontmatter {
                date: Some("2024-01-15".to_string()),
                ..Default::default()
            },
        };

        let file_with_invalid_date = MarkdownFile {
            path: PathBuf::from("invalid_date.md"),
            content: "Content 2".to_string(),
            title: "Invalid Date".to_string(),
            frontmatter: Frontmatter {
                date: Some("not a date".to_string()),
                ..Default::default()
            },
        };

        let mut files = vec![file_with_invalid_date, file_with_valid_date];
        let config = SortConfig {
            field: "date".to_string(),
            order: "asc".to_string(),
        };

        renderer.sort_markdown_files(&mut files, &config);

        // Files with valid dates should come before files with invalid dates
        assert_eq!(files[0].frontmatter.date, Some("2024-01-15".to_string()));
        assert_eq!(files[1].frontmatter.date, Some("not a date".to_string()));
    }

    #[test]
    fn test_compare_by_filename_fallback() {
        let renderer = create_test_html_renderer();

        let file1 = MarkdownFile {
            path: PathBuf::from("z_file.md"),
            content: "Content 1".to_string(),
            title: "Zebra".to_string(),
            frontmatter: Frontmatter::default(),
        };

        let file2 = MarkdownFile {
            path: PathBuf::from("a_file.md"),
            content: "Content 2".to_string(),
            title: "Apple".to_string(),
            frontmatter: Frontmatter::default(),
        };

        let comparison = renderer.compare_by_filename(&file1, &file2);
        assert_eq!(comparison, Ordering::Greater);
    }

    #[test]
    fn test_generate_blog_list_with_sorting() -> Result<()> {
        let temp_dir = tempdir()?;
        let template_manager = TemplateManager::new(temp_dir.path())?;
        let renderer = HtmlRenderer::new(temp_dir.path(), template_manager);

        // Create test files with different dates
        let frontmatter1 = r#"+++
title = "First Post"
date = "2024-01-10"
+++"#;

        let frontmatter2 = r#"+++
title = "Second Post" 
date = "2024-01-15"
+++"#;

        let frontmatter3 = r#"+++
title = "Third Post"
date = "2024-01-05"
+++"#;

        create_test_markdown_file(
            &temp_dir,
            "post1.md",
            frontmatter1,
            "# First Post\nContent here",
        );
        create_test_markdown_file(
            &temp_dir,
            "post2.md",
            frontmatter2,
            "# Second Post\nContent here",
        );
        create_test_markdown_file(
            &temp_dir,
            "post3.md",
            frontmatter3,
            "# Third Post\nContent here",
        );

        // Create list page with sorting configuration
        let list_frontmatter = r#"+++
list = true
title = "Blog"
sort_by = "date"
sort_order = "desc"
+++"#;

        let list_file =
            create_test_markdown_file(&temp_dir, "index.md", list_frontmatter, "# Blog\nWelcome");
        let parsed_list = MarkdownParser::parse_markdown_file(&list_file)?;

        let mut list_pages = HashMap::new();
        list_pages.insert(PathBuf::from(""), &parsed_list);

        // Generate blog list
        let result = renderer.generate_blog_list_content(Path::new(""), &list_pages)?;

        // Verify that posts are in correct order (newest first)
        assert!(result.contains("Second Post"));
        assert!(result.contains("First Post"));
        assert!(result.contains("Third Post"));

        // Check that the order in the result matches expected (newest to oldest)
        let second_index = result.find("Second Post").unwrap_or(0);
        let first_index = result.find("First Post").unwrap_or(0);
        let third_index = result.find("Third Post").unwrap_or(0);

        assert!(second_index < first_index); // Second Post should come before First Post
        assert!(first_index < third_index); // First Post should come before Third Post

        Ok(())
    }

    #[test]
    fn test_html_content_passthrough() {
        let renderer = create_test_html_renderer();
        let html = "<h1>Test</h1><p>Content</p>";
        let result = renderer.process_content(html).unwrap();
        assert_eq!(result, html);
    }

    #[test]
    fn test_markdown_content_conversion() {
        let renderer = create_test_html_renderer();
        let markdown = "# Test\n\nContent here";
        let result = renderer.process_content(markdown).unwrap();
        assert!(result.contains("<h1>Test</h1>"));
        assert!(result.contains("<p>Content here</p>"));
    }

    #[test]
    fn test_unsafe_html_rejection() {
        let renderer = create_test_html_renderer();
        let unsafe_html = "<h1>Test</h1><script>alert('xss')</script>";
        let result = renderer.process_content(unsafe_html);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unsafe element"));
    }

    #[test]
    fn test_html_detection_heuristic() {
        let renderer = create_test_html_renderer();

        // Should detect as HTML
        assert!(renderer.looks_like_html("<h1>Test</h1>"));
        assert!(renderer.looks_like_html("<p>Paragraph</p>"));
        assert!(renderer.looks_like_html("<br/>"));
        assert!(renderer.looks_like_html("   <div>Content</div>")); // with leading whitespace

        // Should not detect as HTML
        assert!(!renderer.looks_like_html("# Markdown heading"));
        assert!(!renderer.looks_like_html("Just plain text"));
        assert!(!renderer.looks_like_html("<unclosed tag")); // no closing tag or self-closing
    }

    #[test]
    fn test_empty_content_handling() {
        let renderer = create_test_html_renderer();

        // Empty string should not be detected as HTML
        assert!(!renderer.looks_like_html(""));

        // But should process without error
        let result = renderer.process_content("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_html_validation_dangerous_elements() {
        let renderer = create_test_html_renderer();

        let dangerous_cases = [
            "<script>alert('xss')</script>",
            "<iframe src='evil.com'></iframe>",
            "<object data='malicious.swf'></object>",
            "<embed src='dangerous content'>",
            "<form action='steal-data.com'></form>",
        ];

        for dangerous_html in &dangerous_cases {
            let result = renderer.validate_basic_html(dangerous_html);
            assert!(result.is_err(), "Should reject: {}", dangerous_html);
        }
    }

    #[test]
    fn test_html_validation_safe_elements() {
        let renderer = create_test_html_renderer();

        let safe_cases = [
            "<h1>Safe heading</h1>",
            "<p>Safe paragraph</p>",
            "<div>Safe div</div>",
            "<span>Safe span</span>",
            "<ul><li>Safe list item</li></ul>",
            "<a href='safe.com'>Safe link</a>",
            "<img src='safe.jpg' alt='Safe image' />",
        ];

        for safe_html in &safe_cases {
            let result = renderer.validate_basic_html(safe_html);
            assert!(result.is_ok(), "Should allow: {}", safe_html);
        }
    }
}
