//! Shared test fixtures for the `nav` submodule tests.

use crate::core::config::SiteConfig;
use crate::core::content::frontmatter::FrontMatter;
use crate::core::content::page::{Page, output_path_for};
use crate::core::nav::href_for;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) fn test_config() -> SiteConfig {
    SiteConfig {
        content_dir: PathBuf::from("content"),
        output_dir: PathBuf::from("_site"),
    }
}

/// `rel` is the source path stem relative to `content/`, without `.md`.
/// Examples: `"about"`, `"blog/post"`, `"index"`, `"blog/index"`. The
/// helper computes source path, output path, URL, and the
/// `is_section_index` flag from this single input — mirroring what
/// `load_page` does.
pub(crate) fn make_page(rel: &str, title: &str) -> Page {
    make_page_with_data(rel, title, gray_matter::Pod::Null)
}

pub(crate) fn make_page_with_data(rel: &str, title: &str, data: gray_matter::Pod) -> Page {
    let config = test_config();
    let source = config.content_dir.join(format!("{}.md", rel));
    let output = output_path_for(&source, &config);
    let url = href_for(&output, &config);
    let is_section_index = Path::new(rel).file_name().and_then(|n| n.to_str()) == Some("index");
    Page {
        frontmatter: FrontMatter {
            title: title.to_string(),
            data,
        },
        content_html: String::new(),
        excerpt_html: None,
        source_path: source,
        output_path: output,
        url,
        is_section_index,
    }
}

pub(crate) fn pod_hash(pairs: &[(&str, gray_matter::Pod)]) -> gray_matter::Pod {
    let mut map = HashMap::new();
    for (k, v) in pairs {
        map.insert(k.to_string(), v.clone());
    }
    gray_matter::Pod::Hash(map)
}
