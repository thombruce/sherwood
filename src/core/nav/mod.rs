use crate::core::config::SiteConfig;
use crate::core::content::page::Page;
use std::path::Path;

mod breadcrumb;
mod url;

#[cfg(test)]
pub(crate) mod test_support;

pub use breadcrumb::Breadcrumb;
pub(crate) use url::href_for;

use breadcrumb::breadcrumbs_for;

#[derive(Debug, Clone)]
pub struct NavItem {
    pub title: String,
    pub href: String,
    pub is_current: bool,
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
    let idx = all_pages
        .iter()
        .position(|p| p.output_path == page.output_path);

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

    PageContext {
        nav,
        breadcrumbs,
        prev,
        next,
        pages: all_pages,
    }
}

/// Nav inclusion rules. By default the top-level nav lists:
///
/// - any top-level page (e.g. `/about/`, `/`)
/// - any section index (`<dir>/.../index.html`) — the landing page of a
///   subdirectory
///
/// Anything else (deep leaf pages like `/blog/first-post/`) is excluded.
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_support::{make_page, make_page_with_data, pod_hash, test_config};

    #[test]
    fn nav_is_current_only_for_page() {
        let config = test_config();
        let pages = vec![make_page("about", "About"), make_page("index", "Home")];
        let ctx = compute_context(&pages[1], &pages, &config);
        assert!(!ctx.nav[0].is_current);
        assert!(ctx.nav[1].is_current);
    }

    #[test]
    fn prev_none_for_first() {
        let config = test_config();
        let pages = vec![make_page("about", "About"), make_page("index", "Home")];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert!(ctx.prev.is_none());
        assert!(ctx.next.is_some());
    }

    #[test]
    fn next_none_for_last() {
        let config = test_config();
        let pages = vec![make_page("about", "About"), make_page("index", "Home")];
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
    fn sort_order() {
        let mut pages = [
            make_page("index", "Home"),
            make_page("about", "About"),
            make_page("blog/post", "Post"),
        ];
        pages.sort_by(|a, b| a.output_path.cmp(&b.output_path));
        assert_eq!(
            pages[0].output_path,
            std::path::PathBuf::from("_site/about/index.html")
        );
        assert_eq!(
            pages[1].output_path,
            std::path::PathBuf::from("_site/blog/post/index.html")
        );
        assert_eq!(
            pages[2].output_path,
            std::path::PathBuf::from("_site/index.html")
        );
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
        let blog: Vec<_> = ctx
            .pages_under("/blog/")
            .iter()
            .map(|p| p.url.clone())
            .collect();
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
        let pages = vec![make_page("index", "Home"), make_page("about", "About")];
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
        let pages = vec![make_page("index", "Home"), make_page("about", "About")];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.pages.len(), 2);
    }
}
