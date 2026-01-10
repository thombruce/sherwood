use super::parsing::MarkdownFile;
use super::sorting::{SortConfig, sort_markdown_files};
use crate::templates::{ListData, ListItemData};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct ListGenerator {
    input_dir: PathBuf,
}

impl ListGenerator {
    pub fn new(input_dir: &Path) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
        }
    }

    /// Generate list data for a directory, collecting and sorting markdown files
    pub fn generate_list_data(
        &self,
        dir: &Path,
        list_pages: &HashMap<PathBuf, &MarkdownFile>,
    ) -> Result<ListData> {
        // Find the list page for this directory to get sort configuration
        let sort_config = list_pages
            .get(dir)
            .map(|list_page| SortConfig::from_frontmatter(&list_page.frontmatter))
            .unwrap_or_else(|| SortConfig {
                field: "date".to_string(),
                order: "desc".to_string(),
            });

        let mut markdown_files = self.collect_markdown_files(dir)?;

        // Sort the collected files
        sort_markdown_files(&mut markdown_files, &sort_config);

        // Convert to ListItemData
        let items = self.convert_to_list_items(markdown_files);

        let total_count = items.len();

        Ok(ListData {
            items,
            sort_config,
            total_count,
        })
    }

    /// Collect all markdown files in a directory (excluding index files)
    fn collect_markdown_files(&self, dir: &Path) -> Result<Vec<MarkdownFile>> {
        let mut markdown_files = Vec::new();

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
                    let parsed = super::parsing::MarkdownParser::parse_markdown_file(&path)?;
                    markdown_files.push(parsed);
                }
            }
        }

        Ok(markdown_files)
    }

    /// Convert MarkdownFile instances to ListItemData
    fn convert_to_list_items(&self, markdown_files: Vec<MarkdownFile>) -> Vec<ListItemData> {
        markdown_files
            .into_iter()
            .map(|parsed| {
                let relative_url_path = parsed
                    .path
                    .strip_prefix(&self.input_dir)
                    .unwrap_or(&parsed.path)
                    .with_extension("");
                let relative_url = relative_url_path.to_string_lossy();

                ListItemData {
                    title: parsed.title.clone(),
                    url: relative_url.to_string(),
                    date: parsed.frontmatter.date.clone(),
                    excerpt: parsed.frontmatter.excerpt.clone(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::parsing::MarkdownParser;
    use std::fs;
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

    #[allow(dead_code)]
    fn create_test_list_generator() -> ListGenerator {
        let temp_dir = tempdir().unwrap();
        ListGenerator::new(temp_dir.path())
    }

    #[test]
    fn test_list_generator_creation() {
        let temp_dir = tempdir().unwrap();
        let generator = ListGenerator::new(temp_dir.path());
        assert_eq!(generator.input_dir, temp_dir.path());
    }

    #[test]
    fn test_generate_blog_list_with_sorting() -> Result<()> {
        let temp_dir = tempdir()?;
        let generator = ListGenerator::new(temp_dir.path());

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

        // Generate list data
        let list_data = generator.generate_list_data(Path::new(""), &list_pages)?;

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
    fn test_list_data_with_excerpt() -> Result<()> {
        let temp_dir = tempdir()?;
        let generator = ListGenerator::new(temp_dir.path());

        // Create test file with excerpt in frontmatter
        let frontmatter = r#"+++
title = "Test Post"
excerpt = "Custom excerpt from frontmatter"
date = "2024-01-15"
+++"#;

        create_test_markdown_file(
            &temp_dir,
            "test.md",
            frontmatter,
            "# Test Post\nContent here",
        );

        // Create list page
        let list_frontmatter = r#"+++
list = true
title = "Blog"
+++"#;

        let list_file =
            create_test_markdown_file(&temp_dir, "index.md", list_frontmatter, "# Blog\nWelcome");
        let parsed_list = MarkdownParser::parse_markdown_file(&list_file)?;

        let mut list_pages = HashMap::new();
        list_pages.insert(PathBuf::from(""), &parsed_list);

        // Generate list data
        let list_data = generator.generate_list_data(Path::new(""), &list_pages)?;

        // Verify excerpt is included
        assert_eq!(list_data.items.len(), 1);
        assert_eq!(
            list_data.items[0].excerpt,
            Some("Custom excerpt from frontmatter".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_list_item_url_generation() -> Result<()> {
        let temp_dir = tempdir()?;
        let generator = ListGenerator::new(temp_dir.path());

        let frontmatter = r#"+++
title = "Test Post"
+++"#;

        // Create file in a subdirectory
        fs::create_dir_all(temp_dir.path().join("posts"))?;
        let file_path = temp_dir.path().join("posts").join("test-post.md");
        let content = format!("{}\n\n# Test Post\nContent here", frontmatter);
        fs::write(&file_path, content)?;

        // Create list page
        let list_frontmatter = r#"+++
list = true
title = "Blog"
+++"#;

        let list_file =
            create_test_markdown_file(&temp_dir, "index.md", list_frontmatter, "# Blog\nWelcome");
        let parsed_list = MarkdownParser::parse_markdown_file(&list_file)?;

        let mut list_pages = HashMap::new();
        list_pages.insert(PathBuf::from("posts"), &parsed_list);

        // Generate list data for posts directory
        let list_data = generator.generate_list_data(Path::new("posts"), &list_pages)?;

        // Verify URL generation
        assert_eq!(list_data.items.len(), 1);
        assert_eq!(list_data.items[0].url, "posts/test-post");

        Ok(())
    }

    #[test]
    fn test_skip_index_files() -> Result<()> {
        let temp_dir = tempdir()?;
        let generator = ListGenerator::new(temp_dir.path());

        // Create multiple index files that should be skipped
        let index_frontmatter = r#"+++
title = "Index"
+++"#;

        create_test_markdown_file(&temp_dir, "index.md", index_frontmatter, "# Index");
        create_test_markdown_file(&temp_dir, "index2.md", index_frontmatter, "# Index 2");
        create_test_markdown_file(
            &temp_dir,
            "index_special.md",
            index_frontmatter,
            "# Index Special",
        );

        // Create a regular content file that should be included
        let content_frontmatter = r#"+++
title = "Content File"
+++"#;

        create_test_markdown_file(&temp_dir, "content.md", content_frontmatter, "# Content");

        // Create list page
        let list_frontmatter = r#"+++
list = true
title = "Blog"
+++"#;

        let list_file =
            create_test_markdown_file(&temp_dir, "index.md", list_frontmatter, "# List\nWelcome");
        let parsed_list = MarkdownParser::parse_markdown_file(&list_file)?;

        let mut list_pages = HashMap::new();
        list_pages.insert(PathBuf::from(""), &parsed_list);

        // Generate list data
        let list_data = generator.generate_list_data(Path::new(""), &list_pages)?;

        // Only the content file should be included
        assert_eq!(list_data.items.len(), 1);
        assert_eq!(list_data.items[0].title, "Content File");

        Ok(())
    }

    #[test]
    fn test_default_sort_config() -> Result<()> {
        let temp_dir = tempdir()?;
        let generator = ListGenerator::new(temp_dir.path());

        let frontmatter = r#"+++
title = "Test Post"
date = "2024-01-15"
+++"#;

        create_test_markdown_file(&temp_dir, "test.md", frontmatter, "# Test Post");

        // Generate list data without a list page (should use defaults)
        let list_data = generator.generate_list_data(Path::new(""), &HashMap::new())?;

        // Verify default sort configuration
        assert_eq!(list_data.sort_config.field, "date");
        assert_eq!(list_data.sort_config.order, "desc");

        Ok(())
    }
}
