use std::path::{Path, PathBuf};
use crate::config::SiteConfig;
use crate::page::Page;

#[derive(Debug, Clone)]
pub struct NavItem {
    pub title: String,
    pub href: String,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub title: String,
    pub href: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PageContext<'a> {
    pub nav: Vec<NavItem>,
    pub breadcrumbs: Vec<Breadcrumb>,
    pub prev: Option<NavItem>,
    pub next: Option<NavItem>,
    /// All pages in the site, in build order (root index first, then by
    /// output path). Templates can iterate, filter, and sort this to build
    /// indexes, archives, tag listings, etc.
    pub pages: &'a [Page],
}

impl<'a> PageContext<'a> {
    /// Pages whose URL starts with the given prefix. Use to drive section
    /// indexes — e.g. a `/blog/index.html` page can call
    /// `ctx.pages_under("/blog/")` to list every post under `blog/`.
    /// The current page is included; filter it out yourself if undesired.
    pub fn pages_under(&self, url_prefix: &str) -> Vec<&'a Page> {
        self.pages
            .iter()
            .filter(|p| p.url.starts_with(url_prefix))
            .collect()
    }
}

pub fn compute_context<'a>(
    page: &Page,
    all_pages: &'a [Page],
    config: &SiteConfig,
) -> PageContext<'a> {
    let idx = all_pages.iter().position(|p| p.output_path == page.output_path);

    let nav = all_pages
        .iter()
        .filter(|p| include_in_nav(p, config))
        .map(|p| nav_item_for(p, p.output_path == page.output_path))
        .collect();

    let prev = idx
        .filter(|&i| i > 0)
        .map(|i| nav_item_for(&all_pages[i - 1], false));

    let next = idx
        .filter(|&i| i + 1 < all_pages.len())
        .map(|i| nav_item_for(&all_pages[i + 1], false));

    let breadcrumbs = breadcrumbs_for(page, all_pages, config);

    PageContext { nav, breadcrumbs, prev, next, pages: all_pages }
}

/// Nav inclusion rules. By default the top-level nav lists:
/// - any top-level page (e.g. `/about.html`, `/index.html`)
/// - any section index (`<dir>/.../index.html`) — the landing page of a
///   subdirectory
/// Anything else (deep leaf pages like `/blog/first-post.html`) is excluded.
///
/// Frontmatter `nav: true` force-includes a page that wouldn't otherwise
/// qualify; `nav: false` force-excludes one that would.
fn include_in_nav(page: &Page, config: &SiteConfig) -> bool {
    if let Some(gray_matter::Pod::Boolean(b)) = page.frontmatter.get("nav") {
        return *b;
    }
    if page.is_section_index {
        return true;
    }
    let relative = page
        .source_path
        .strip_prefix(&config.content_dir)
        .unwrap_or(&page.source_path);
    let normal_components: Vec<_> = relative
        .components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .collect();
    normal_components.len() <= 1
}

pub(crate) fn is_root_index(page: &Page, config: &SiteConfig) -> bool {
    page.output_path
        .strip_prefix(&config.output_dir)
        .map(|r| r == Path::new("index.html"))
        .unwrap_or(false)
}

fn nav_item_for(p: &Page, is_current: bool) -> NavItem {
    NavItem {
        title: p.frontmatter.title.clone(),
        href: p.url.clone(),
        is_current,
    }
}

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

fn breadcrumbs_for(page: &Page, all_pages: &[Page], config: &SiteConfig) -> Vec<Breadcrumb> {
    let relative = page
        .output_path
        .strip_prefix(&config.output_dir)
        .unwrap_or(&page.output_path);

    if relative == Path::new("index.html") {
        return vec![];
    }

    let mut crumbs = Vec::new();

    let home_title = all_pages
        .iter()
        .find(|p| is_root_index(p, config))
        .map(|p| p.frontmatter.title.clone())
        .unwrap_or_else(|| "Home".to_string());

    crumbs.push(Breadcrumb {
        title: home_title,
        href: Some("/".to_string()),
    });

    let components: Vec<_> = relative.components().collect();
    let num_dirs = components.len().saturating_sub(1);
    let mut path_so_far = PathBuf::new();

    for comp in components.iter().take(num_dirs) {
        let dir_name = match comp {
            std::path::Component::Normal(s) => s.to_string_lossy().into_owned(),
            _ => continue,
        };
        path_so_far.push(&dir_name);

        let index_relative = path_so_far.join("index.html");
        let index_output = config.output_dir.join(&index_relative);

        let dir_title = all_pages
            .iter()
            .find(|p| p.output_path == index_output)
            .map(|p| p.frontmatter.title.clone())
            .unwrap_or_else(|| capitalize_first(&dir_name));

        crumbs.push(Breadcrumb {
            title: dir_title,
            href: Some(href_for(&index_output, config)),
        });
    }

    let is_dir_index = relative.file_name().and_then(|n| n.to_str()) == Some("index.html")
        && relative.components().count() > 1;

    if is_dir_index {
        // The leaf is `<dir>/index.html` — its parent dir crumb already names
        // this page, so don't append a duplicate. Just unlink the dir crumb so
        // it renders as the current location.
        if let Some(last) = crumbs.last_mut() {
            last.href = None;
        }
    } else {
        crumbs.push(Breadcrumb {
            title: page.frontmatter.title.clone(),
            href: None,
        });
    }

    crumbs
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontmatter::FrontMatter;
    use crate::page::output_path_for;

    fn test_config() -> SiteConfig {
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
    fn make_page(rel: &str, title: &str) -> Page {
        make_page_with_data(rel, title, gray_matter::Pod::Null)
    }

    fn make_page_with_data(rel: &str, title: &str, data: gray_matter::Pod) -> Page {
        let config = test_config();
        let source = config.content_dir.join(format!("{}.md", rel));
        let output = output_path_for(&source, &config);
        let url = href_for(&output, &config);
        let is_section_index =
            Path::new(rel).file_name().and_then(|n| n.to_str()) == Some("index");
        Page {
            frontmatter: FrontMatter { title: title.to_string(), data },
            content_html: String::new(),
            excerpt_html: None,
            source_path: source,
            output_path: output,
            url,
            is_section_index,
        }
    }

    fn pod_hash(pairs: &[(&str, gray_matter::Pod)]) -> gray_matter::Pod {
        let mut map = std::collections::HashMap::new();
        for (k, v) in pairs {
            map.insert(k.to_string(), v.clone());
        }
        gray_matter::Pod::Hash(map)
    }

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
    fn nav_is_current_only_for_page() {
        let config = test_config();
        let pages = vec![
            make_page("about", "About"),
            make_page("index", "Home"),
        ];
        let ctx = compute_context(&pages[1], &pages, &config);
        assert!(!ctx.nav[0].is_current);
        assert!(ctx.nav[1].is_current);
    }

    #[test]
    fn prev_none_for_first() {
        let config = test_config();
        let pages = vec![
            make_page("about", "About"),
            make_page("index", "Home"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert!(ctx.prev.is_none());
        assert!(ctx.next.is_some());
    }

    #[test]
    fn next_none_for_last() {
        let config = test_config();
        let pages = vec![
            make_page("about", "About"),
            make_page("index", "Home"),
        ];
        let ctx = compute_context(&pages[1], &pages, &config);
        assert!(ctx.prev.is_some());
        assert!(ctx.next.is_none());
    }

    #[test]
    fn prev_next_for_middle() {
        let config = test_config();
        let pages = vec![
            make_page("about", "About"),
            make_page("blog/post", "Post"),
            make_page("index", "Home"),
        ];
        let ctx = compute_context(&pages[1], &pages, &config);
        assert!(ctx.prev.is_some());
        assert!(ctx.next.is_some());
    }

    #[test]
    fn only_page_has_no_prev_next() {
        let config = test_config();
        let pages = vec![make_page("index", "Home")];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert!(ctx.prev.is_none());
        assert!(ctx.next.is_none());
    }

    #[test]
    fn breadcrumbs_empty_for_root() {
        let config = test_config();
        let pages = vec![make_page("index", "Home")];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert!(ctx.breadcrumbs.is_empty());
    }

    #[test]
    fn breadcrumbs_flat_includes_home() {
        let config = test_config();
        let pages = vec![
            make_page("about", "About"),
            make_page("index", "Home"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs.len(), 2);
        assert_eq!(ctx.breadcrumbs[0].title, "Home");
        assert_eq!(ctx.breadcrumbs[0].href.as_deref(), Some("/"));
        assert_eq!(ctx.breadcrumbs[1].title, "About");
        assert!(ctx.breadcrumbs[1].href.is_none());
    }

    #[test]
    fn breadcrumbs_nested_has_dir_crumb() {
        let config = test_config();
        let pages = vec![
            make_page("blog/post", "My Post"),
            make_page("index", "Home"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs.len(), 3);
        assert_eq!(ctx.breadcrumbs[0].title, "Home");
        assert_eq!(ctx.breadcrumbs[1].title, "Blog");
        assert_eq!(ctx.breadcrumbs[1].href.as_deref(), Some("/blog/"));
        assert_eq!(ctx.breadcrumbs[2].title, "My Post");
        assert!(ctx.breadcrumbs[2].href.is_none());
    }

    #[test]
    fn sort_order() {
        let mut pages = [
            make_page("index", "Home"),
            make_page("about", "About"),
            make_page("blog/post", "Post"),
        ];
        pages.sort_by(|a, b| a.output_path.cmp(&b.output_path));
        assert_eq!(pages[0].output_path, PathBuf::from("_site/about/index.html"));
        assert_eq!(pages[1].output_path, PathBuf::from("_site/blog/post/index.html"));
        assert_eq!(pages[2].output_path, PathBuf::from("_site/index.html"));
    }

    #[test]
    fn breadcrumbs_depth_3() {
        let config = test_config();
        let pages = vec![
            make_page("a/b/c", "C Page"),
            make_page("index", "Home"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs.len(), 4);
        assert_eq!(ctx.breadcrumbs[0].title, "Home");
        assert_eq!(ctx.breadcrumbs[1].title, "A");
        assert_eq!(ctx.breadcrumbs[2].title, "B");
        assert_eq!(ctx.breadcrumbs[3].title, "C Page");
        assert!(ctx.breadcrumbs[3].href.is_none());
    }

    #[test]
    fn home_crumb_uses_index_page_title() {
        let config = test_config();
        let pages = vec![
            make_page("about", "About"),
            make_page("index", "Welcome"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs[0].title, "Welcome");
    }

    #[test]
    fn home_crumb_defaults_when_no_root_index() {
        let config = test_config();
        let pages = vec![make_page("about", "About")];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs[0].title, "Home");
    }

    #[test]
    fn is_root_index_detects_root() {
        let config = test_config();
        let root = make_page("index", "Home");
        let nested = make_page("blog/index", "Blog");
        let other = make_page("about", "About");
        assert!(is_root_index(&root, &config));
        assert!(!is_root_index(&nested, &config));
        assert!(!is_root_index(&other, &config));
    }

    #[test]
    fn breadcrumbs_dir_index_no_duplicate_leaf() {
        let config = test_config();
        let pages = vec![
            make_page("blog/index", "Blog"),
            make_page("index", "Home"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs.len(), 2);
        assert_eq!(ctx.breadcrumbs[0].title, "Home");
        assert_eq!(ctx.breadcrumbs[1].title, "Blog");
        assert!(ctx.breadcrumbs[1].href.is_none());
    }

    #[test]
    fn pages_under_filters_by_url_prefix() {
        let config = test_config();
        let pages = vec![
            make_page("index", "Home"),
            make_page("about", "About"),
            make_page("blog/index", "Blog"),
            make_page("blog/first", "First"),
            make_page("blog/second", "Second"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        let blog: Vec<_> = ctx.pages_under("/blog/").iter().map(|p| p.url.clone()).collect();
        assert_eq!(blog, vec!["/blog/", "/blog/first/", "/blog/second/"]);
    }

    #[test]
    fn pages_under_empty_for_unknown_prefix() {
        let config = test_config();
        let pages = vec![make_page("index", "Home")];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert!(ctx.pages_under("/nope/").is_empty());
    }

    #[test]
    fn nav_includes_top_level_pages() {
        let config = test_config();
        let pages = vec![
            make_page("index", "Home"),
            make_page("about", "About"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.nav.len(), 2);
    }

    #[test]
    fn nav_includes_section_indexes_excludes_leaves() {
        let config = test_config();
        let pages = vec![
            make_page("index", "Home"),
            make_page("blog/index", "Blog"),
            make_page("blog/post", "Post"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        let titles: Vec<_> = ctx.nav.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(titles, vec!["Home", "Blog"]);
    }

    #[test]
    fn nav_false_hides_top_level_page() {
        let config = test_config();
        let pages = vec![
            make_page("index", "Home"),
            make_page_with_data(
                "private",
                "Private",
                pod_hash(&[("nav", gray_matter::Pod::Boolean(false))]),
            ),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        let titles: Vec<_> = ctx.nav.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(titles, vec!["Home"]);
    }

    #[test]
    fn nav_true_force_includes_leaf() {
        let config = test_config();
        let pages = vec![
            make_page("index", "Home"),
            make_page_with_data(
                "blog/featured",
                "Featured",
                pod_hash(&[("nav", gray_matter::Pod::Boolean(true))]),
            ),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        let titles: Vec<_> = ctx.nav.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(titles, vec!["Home", "Featured"]);
    }

    #[test]
    fn context_exposes_full_pages_slice() {
        let config = test_config();
        let pages = vec![
            make_page("index", "Home"),
            make_page("about", "About"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.pages.len(), 2);
    }
}
