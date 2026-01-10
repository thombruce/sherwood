use super::parsing::MarkdownFile;
use crate::content::date_parsing::parse_date;
use serde::Serialize;
use std::cmp::Ordering;

#[derive(Debug, Serialize, Clone)]
pub struct SortConfig {
    pub field: String,
    pub order: String,
}

impl SortConfig {
    pub fn from_frontmatter(frontmatter: &super::parsing::Frontmatter) -> Self {
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

    pub fn is_valid_field(field: &str) -> bool {
        matches!(field, "date" | "title" | "filename")
    }

    pub fn is_valid_order(order: &str) -> bool {
        matches!(order, "asc" | "desc")
    }

    pub fn validate_and_get_field(&self) -> &str {
        if SortConfig::is_valid_field(&self.field) {
            &self.field
        } else {
            eprintln!(
                "Warning: Invalid sort field '{}', falling back to 'date'",
                self.field
            );
            "date"
        }
    }

    pub fn validate_and_get_order(&self) -> &str {
        if SortConfig::is_valid_order(&self.order) {
            &self.order
        } else {
            eprintln!(
                "Warning: Invalid sort order '{}', falling back to 'asc'",
                self.order
            );
            "asc"
        }
    }
}

/// Sort markdown files according to the provided configuration
pub fn sort_markdown_files(files: &mut [MarkdownFile], sort_config: &SortConfig) {
    let field = sort_config.validate_and_get_field();
    let order = sort_config.validate_and_get_order();

    files.sort_by(|a, b| {
        let comparison = match field {
            "date" => compare_by_date(a, b),
            "title" => a.title.cmp(&b.title),
            "filename" => compare_by_filename(a, b),
            _ => Ordering::Equal, // Should not reach here due to validation
        };

        if order == "desc" {
            comparison.reverse()
        } else {
            comparison
        }
    });
}

/// Compare two markdown files by their dates
fn compare_by_date(a: &MarkdownFile, b: &MarkdownFile) -> Ordering {
    match (&a.frontmatter.date, &b.frontmatter.date) {
        (Some(date_a), Some(date_b)) => {
            match (parse_date(date_a), parse_date(date_b)) {
                (Some(parsed_a), Some(parsed_b)) => parsed_a.cmp(&parsed_b),
                (Some(_), None) => Ordering::Less, // Valid date comes before invalid
                (None, Some(_)) => Ordering::Greater,
                (None, None) => compare_by_filename(a, b), // Both invalid, fall back to filename
            }
        }
        (Some(_), None) => Ordering::Less, // File with date comes before file without
        (None, Some(_)) => Ordering::Greater,
        (None, None) => compare_by_filename(a, b), // Neither has date, fall back to filename
    }
}

/// Compare two markdown files by their filenames
fn compare_by_filename(a: &MarkdownFile, b: &MarkdownFile) -> Ordering {
    a.path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .cmp(b.path.file_name().and_then(|n| n.to_str()).unwrap_or(""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::parsing::{Frontmatter, MarkdownFile};
    use std::path::PathBuf;

    fn create_test_markdown_file(path: &str, title: &str, date: Option<&str>) -> MarkdownFile {
        MarkdownFile {
            path: PathBuf::from(path),
            content: "Content".to_string(),
            title: title.to_string(),
            frontmatter: Frontmatter {
                date: date.map(|d| d.to_string()),
                ..Default::default()
            },
        }
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
    fn test_sort_config_validation() {
        let config = SortConfig {
            field: "invalid".to_string(),
            order: "invalid".to_string(),
        };

        assert_eq!(config.validate_and_get_field(), "date");
        assert_eq!(config.validate_and_get_order(), "asc");
    }

    #[test]
    fn test_sort_by_date_ascending() {
        let file1 = create_test_markdown_file("file1.md", "File 1", Some("2024-01-10"));
        let file2 = create_test_markdown_file("file2.md", "File 2", Some("2024-01-15"));

        let mut files = vec![file2, file1];
        let config = SortConfig {
            field: "date".to_string(),
            order: "asc".to_string(),
        };

        sort_markdown_files(&mut files, &config);

        assert_eq!(files[0].frontmatter.date, Some("2024-01-10".to_string()));
        assert_eq!(files[1].frontmatter.date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_sort_by_date_descending() {
        let file1 = create_test_markdown_file("file1.md", "File 1", Some("2024-01-10"));
        let file2 = create_test_markdown_file("file2.md", "File 2", Some("2024-01-15"));

        let mut files = vec![file1, file2];
        let config = SortConfig {
            field: "date".to_string(),
            order: "desc".to_string(),
        };

        sort_markdown_files(&mut files, &config);

        assert_eq!(files[0].frontmatter.date, Some("2024-01-15".to_string()));
        assert_eq!(files[1].frontmatter.date, Some("2024-01-10".to_string()));
    }

    #[test]
    fn test_sort_by_title() {
        let file1 = create_test_markdown_file("z_file.md", "Zebra", None);
        let file2 = create_test_markdown_file("a_file.md", "Apple", None);

        let mut files = vec![file1, file2];
        let config = SortConfig {
            field: "title".to_string(),
            order: "asc".to_string(),
        };

        sort_markdown_files(&mut files, &config);

        assert_eq!(files[0].title, "Apple");
        assert_eq!(files[1].title, "Zebra");
    }

    #[test]
    fn test_sort_by_filename() {
        let file1 = create_test_markdown_file("z_file.md", "Zebra", None);
        let file2 = create_test_markdown_file("a_file.md", "Apple", None);

        let mut files = vec![file1, file2];
        let config = SortConfig {
            field: "filename".to_string(),
            order: "asc".to_string(),
        };

        sort_markdown_files(&mut files, &config);

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
        let file_with_date =
            create_test_markdown_file("with_date.md", "With Date", Some("2024-01-15"));
        let file_without_date = create_test_markdown_file("without_date.md", "Without Date", None);

        let mut files = vec![file_without_date, file_with_date];
        let config = SortConfig {
            field: "date".to_string(),
            order: "asc".to_string(),
        };

        sort_markdown_files(&mut files, &config);

        // Files with dates should come before files without dates
        assert_eq!(files[0].frontmatter.date, Some("2024-01-15".to_string()));
        assert_eq!(files[1].frontmatter.date, None);
    }

    #[test]
    fn test_sort_with_invalid_dates() {
        let file_with_valid_date =
            create_test_markdown_file("valid_date.md", "Valid Date", Some("2024-01-15"));
        let file_with_invalid_date =
            create_test_markdown_file("invalid_date.md", "Invalid Date", Some("not a date"));

        let mut files = vec![file_with_invalid_date, file_with_valid_date];
        let config = SortConfig {
            field: "date".to_string(),
            order: "asc".to_string(),
        };

        sort_markdown_files(&mut files, &config);

        // Files with valid dates should come before files with invalid dates
        assert_eq!(files[0].frontmatter.date, Some("2024-01-15".to_string()));
        assert_eq!(files[1].frontmatter.date, Some("not a date".to_string()));
    }

    #[test]
    fn test_compare_by_filename_fallback() {
        let file1 = create_test_markdown_file("z_file.md", "Zebra", None);
        let file2 = create_test_markdown_file("a_file.md", "Apple", None);

        let comparison = compare_by_filename(&file1, &file2);
        assert_eq!(comparison, Ordering::Greater);
    }
}
