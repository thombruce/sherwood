use super::list_generation::ListGenerator;
use crate::templates::TemplateManager;
use anyhow::Result;
use std::path::Path;

pub use super::sorting::SortConfig;

/// Simplified HTML renderer that delegates to specialized modules
pub struct HtmlRenderer {
    #[allow(dead_code)] // Currently unused but kept for future functionality
    input_dir: std::path::PathBuf,
    list_generator: ListGenerator,
}

impl HtmlRenderer {
    pub fn new(input_dir: &Path, _template_manager: TemplateManager) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            list_generator: ListGenerator::new(input_dir),
        }
    }

    /// Process content - delegates to validation module
    pub fn process_content(&self, content: &str) -> Result<String> {
        super::validation::process_content(content)
    }

    /// Generate list data - delegates to list generation module
    pub fn generate_list_data(
        &self,
        dir: &Path,
        list_pages: &std::collections::HashMap<std::path::PathBuf, &super::parsing::MarkdownFile>,
    ) -> Result<crate::templates::ListData> {
        self.list_generator.generate_list_data(dir, list_pages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::parsing::MarkdownParser;
    use crate::templates::TemplateManager;
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
        let temp_dir = tempdir().unwrap();
        let template_manager = TemplateManager::new(temp_dir.path()).unwrap();
        HtmlRenderer::new(temp_dir.path(), template_manager)
    }

    #[test]
    fn test_html_content_passthrough() {
        let renderer = create_test_html_renderer();
        let html = "<h1>Test</h1><p>Content here</p>";
        let result = renderer.process_content(html).unwrap();
        assert_eq!(result, html); // HTML should pass through unchanged
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
    fn test_empty_content_handling() {
        let renderer = create_test_html_renderer();

        // Should process empty string without error
        let result = renderer.process_content("").unwrap();
        assert_eq!(result, "");
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

        let mut list_pages = std::collections::HashMap::new();
        list_pages.insert(PathBuf::from(""), &parsed_list);

        // Generate list data
        let list_data = renderer.generate_list_data(Path::new(""), &list_pages)?;

        // Verify that we have the expected number of items
        assert_eq!(list_data.items.len(), 3);
        assert_eq!(list_data.total_count, 3);

        // Verify that posts are in correct order (newest first)
        assert!(list_data.items[0].title.contains("Second Post"));
        assert!(list_data.items[1].title.contains("First Post"));
        assert!(list_data.items[2].title.contains("Third Post"));

        // Verify sort configuration
        assert_eq!(list_data.sort_config.field, "date");
        assert_eq!(list_data.sort_config.order, "desc");

        Ok(())
    }

    #[test]
    fn test_renderer_with_frontmatter_excerpt() -> Result<()> {
        let temp_dir = tempdir()?;
        let template_manager = TemplateManager::new(temp_dir.path())?;
        let _renderer = HtmlRenderer::new(temp_dir.path(), template_manager);

        // Create test file with excerpt in frontmatter
        let frontmatter = r#"+++
title = "Test Post"
excerpt = "Custom excerpt from frontmatter"
date = "2024-01-15"
+++"#;

        let file_path = temp_dir.path().join("test.md");
        let content = format!(
            "{}\n\n# Test Post\n\nFirst paragraph.\n\nSecond paragraph.",
            frontmatter
        );
        fs::write(&file_path, content)?;

        let parsed = MarkdownParser::parse_markdown_file(&file_path)?;

        // Verify that the parser set the excerpt correctly
        assert_eq!(
            parsed.frontmatter.excerpt,
            Some("Custom excerpt from frontmatter".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_renderer_with_extracted_excerpt() -> Result<()> {
        let temp_dir = tempdir()?;
        let template_manager = TemplateManager::new(temp_dir.path())?;
        let _renderer = HtmlRenderer::new(temp_dir.path(), template_manager);

        // Create test file without excerpt in frontmatter (should be extracted)
        let frontmatter = r#"+++
title = "Test Post"
date = "2024-01-15"
+++"#;

        let file_path = temp_dir.path().join("test.md");
        let content = format!(
            "{}\n\n# Test Post\n\nThis excerpt should be extracted.\n\nSecond paragraph.",
            frontmatter
        );
        fs::write(&file_path, content)?;

        let parsed = MarkdownParser::parse_markdown_file(&file_path)?;

        // Verify that the parser extracted the excerpt
        assert_eq!(
            parsed.frontmatter.excerpt,
            Some("This excerpt should be extracted.".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_renderer_with_no_excerpt() -> Result<()> {
        let temp_dir = tempdir()?;
        let template_manager = TemplateManager::new(temp_dir.path())?;
        let _renderer = HtmlRenderer::new(temp_dir.path(), template_manager);

        // Create test file without excerpt and with no paragraphs (no excerpt possible)
        let frontmatter = r#"+++
title = "Test Post"
date = "2024-01-15"
+++"#;

        let file_path = temp_dir.path().join("test.md");
        let content = format!("{}\n\n# Just a heading", frontmatter);
        fs::write(&file_path, content)?;

        let parsed = MarkdownParser::parse_markdown_file(&file_path)?;

        // Verify that no excerpt was extracted
        assert_eq!(parsed.frontmatter.excerpt, None);

        Ok(())
    }

    #[test]
    fn test_renderer_excerpt_priority_frontmatter() -> Result<()> {
        let temp_dir = tempdir()?;
        let template_manager = TemplateManager::new(temp_dir.path())?;
        let _renderer = HtmlRenderer::new(temp_dir.path(), template_manager);

        // Create test file with excerpt in frontmatter AND content that could be extracted
        let frontmatter = r#"+++
title = "Test Post"
excerpt = "Priority excerpt"
date = "2024-01-15"
+++"#;

        let file_path = temp_dir.path().join("test.md");
        let content = format!(
            "{}\n\n# Test Post\n\nThis should NOT be extracted.\n\nSecond paragraph.",
            frontmatter
        );
        fs::write(&file_path, content)?;

        let parsed = MarkdownParser::parse_markdown_file(&file_path)?;

        // Verify frontmatter excerpt takes priority
        assert_eq!(
            parsed.frontmatter.excerpt,
            Some("Priority excerpt".to_string())
        );

        Ok(())
    }
}
