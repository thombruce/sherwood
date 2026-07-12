use crate::core::config::SiteConfig;
use std::path::Path;

pub(crate) fn href_for(output_path: &Path, config: &SiteConfig) -> String {
    let relative = output_path
        .strip_prefix(&config.output_dir)
        .unwrap_or(output_path);
    let url_path = if relative.file_name().and_then(|n| n.to_str()) == Some("index.html") {
        relative.parent().unwrap_or(Path::new(""))
    } else {
        relative
    };
    let url = path_to_url(url_path);
    if url == "/" {
        return url;
    }
    if url.ends_with('/') {
        url
    } else {
        format!("{}/", url)
    }
}

/// The section a canonical URL belongs to: its parent directory. `/blog/first/`
/// → `/blog/`; `/about/` and section indexes like `/blog/` → `/`; the root `/`
/// → `/`. Drives prev/next scoping — pages chain only within their section.
pub(crate) fn section_of(url: &str) -> &str {
    let trimmed = url.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(i) => &trimmed[..=i],
        None => "/",
    }
}

/// Prefix a canonical (root-relative) URL with the configured base path,
/// turning `/guide/` into `/sherwood/guide/`. An empty base is the identity.
/// The site root `/` resolves to `<base>/`.
pub(crate) fn resolve(canonical: &str, base: &str) -> String {
    if base.is_empty() {
        canonical.to_string()
    } else if canonical == "/" {
        format!("{base}/")
    } else {
        format!("{base}{canonical}")
    }
}

// Build an absolute URL from a relative output path. We walk components and
// join with '/' rather than using `Path::display()` because on Windows
// `display()` would emit '\' separators, producing invalid URLs like
// "/blog\post.html".
pub(crate) fn path_to_url(relative: &Path) -> String {
    let segments: Vec<String> = relative
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    format!("/{}", segments.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nav::test_support::{make_page, test_config};
    use std::path::PathBuf;

    #[test]
    fn href_flat() {
        let config = test_config();
        let page = make_page("about", "About");
        assert_eq!(href_for(&page.output_path, &config), "/about/");
    }

    #[test]
    fn href_nested() {
        let config = test_config();
        let page = make_page("blog/post", "Post");
        assert_eq!(href_for(&page.output_path, &config), "/blog/post/");
    }

    #[test]
    fn href_root_index() {
        let config = test_config();
        let page = make_page("index", "Home");
        assert_eq!(href_for(&page.output_path, &config), "/");
    }

    #[test]
    fn href_section_index() {
        let config = test_config();
        let page = make_page("blog/index", "Blog");
        assert_eq!(href_for(&page.output_path, &config), "/blog/");
    }

    #[test]
    fn path_to_url_joins_with_forward_slash() {
        let p = PathBuf::from("a").join("b").join("c.html");
        assert_eq!(path_to_url(&p), "/a/b/c.html");
    }

    #[test]
    fn section_of_groups_by_parent_dir() {
        assert_eq!(section_of("/blog/first/"), "/blog/");
        assert_eq!(section_of("/guide/advanced/x/"), "/guide/advanced/");
        // Top-level pages, section indexes, and the root all live in "/".
        assert_eq!(section_of("/about/"), "/");
        assert_eq!(section_of("/blog/"), "/");
        assert_eq!(section_of("/"), "/");
    }

    #[test]
    fn resolve_empty_base_is_identity() {
        assert_eq!(resolve("/guide/", ""), "/guide/");
        assert_eq!(resolve("/", ""), "/");
    }

    #[test]
    fn resolve_prefixes_base() {
        assert_eq!(resolve("/guide/", "/sherwood"), "/sherwood/guide/");
        assert_eq!(resolve("/", "/sherwood"), "/sherwood/");
        assert_eq!(resolve("/style.css", "/sherwood"), "/sherwood/style.css");
    }
}
