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
pub struct PageContext {
    pub nav: Vec<NavItem>,
    pub breadcrumbs: Vec<Breadcrumb>,
    pub prev: Option<NavItem>,
    pub next: Option<NavItem>,
}

pub fn compute_context(page: &Page, all_pages: &[Page], config: &SiteConfig) -> PageContext {
    let idx = all_pages.iter().position(|p| p.output_path == page.output_path);

    let nav = all_pages
        .iter()
        .map(|p| NavItem {
            title: p.frontmatter.title.clone(),
            href: href_for(&p.output_path, config),
            is_current: p.output_path == page.output_path,
        })
        .collect();

    let prev = idx.filter(|&i| i > 0).map(|i| {
        let p = &all_pages[i - 1];
        NavItem {
            title: p.frontmatter.title.clone(),
            href: href_for(&p.output_path, config),
            is_current: false,
        }
    });

    let next = idx.filter(|&i| i + 1 < all_pages.len()).map(|i| {
        let p = &all_pages[i + 1];
        NavItem {
            title: p.frontmatter.title.clone(),
            href: href_for(&p.output_path, config),
            is_current: false,
        }
    });

    let breadcrumbs = breadcrumbs_for(page, all_pages, config);

    PageContext { nav, breadcrumbs, prev, next }
}

fn href_for(output_path: &Path, config: &SiteConfig) -> String {
    let relative = output_path
        .strip_prefix(&config.output_dir)
        .unwrap_or(output_path);
    format!("/{}", relative.display())
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

    // Home crumb — use actual title if root index exists
    let home_title = all_pages
        .iter()
        .find(|p| {
            p.output_path
                .strip_prefix(&config.output_dir)
                .map(|r| r == Path::new("index.html"))
                .unwrap_or(false)
        })
        .map(|p| p.frontmatter.title.clone())
        .unwrap_or_else(|| "Home".to_string());

    crumbs.push(Breadcrumb {
        title: home_title,
        href: Some("/index.html".to_string()),
    });

    // Intermediate directory segments
    let components: Vec<_> = relative.components().collect();
    let num_dirs = components.len().saturating_sub(1);
    let mut path_so_far = PathBuf::new();

    for i in 0..num_dirs {
        let dir_name = match components[i] {
            std::path::Component::Normal(s) => s.to_string_lossy().to_string(),
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
            href: Some(format!("/{}", index_relative.display())),
        });
    }

    // Current page — no link
    crumbs.push(Breadcrumb {
        title: page.frontmatter.title.clone(),
        href: None,
    });

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

    fn test_config() -> SiteConfig {
        SiteConfig {
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("_site"),
        }
    }

    fn make_page(output: &str, title: &str) -> Page {
        Page {
            frontmatter: FrontMatter { title: title.to_string() },
            content_html: String::new(),
            source_path: PathBuf::from(output),
            output_path: PathBuf::from(output),
        }
    }

    #[test]
    fn href_flat() {
        let config = test_config();
        let page = make_page("_site/about.html", "About");
        assert_eq!(href_for(&page.output_path, &config), "/about.html");
    }

    #[test]
    fn href_nested() {
        let config = test_config();
        let page = make_page("_site/blog/post.html", "Post");
        assert_eq!(href_for(&page.output_path, &config), "/blog/post.html");
    }

    #[test]
    fn nav_is_current_only_for_page() {
        let config = test_config();
        let pages = vec![
            make_page("_site/about.html", "About"),
            make_page("_site/blog/post.html", "Post"),
            make_page("_site/index.html", "Home"),
        ];
        let ctx = compute_context(&pages[1], &pages, &config);
        assert!(!ctx.nav[0].is_current);
        assert!(ctx.nav[1].is_current);
        assert!(!ctx.nav[2].is_current);
    }

    #[test]
    fn prev_none_for_first() {
        let config = test_config();
        let pages = vec![
            make_page("_site/about.html", "About"),
            make_page("_site/index.html", "Home"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert!(ctx.prev.is_none());
        assert!(ctx.next.is_some());
    }

    #[test]
    fn next_none_for_last() {
        let config = test_config();
        let pages = vec![
            make_page("_site/about.html", "About"),
            make_page("_site/index.html", "Home"),
        ];
        let ctx = compute_context(&pages[1], &pages, &config);
        assert!(ctx.prev.is_some());
        assert!(ctx.next.is_none());
    }

    #[test]
    fn prev_next_for_middle() {
        let config = test_config();
        let pages = vec![
            make_page("_site/about.html", "About"),
            make_page("_site/blog/post.html", "Post"),
            make_page("_site/index.html", "Home"),
        ];
        let ctx = compute_context(&pages[1], &pages, &config);
        assert!(ctx.prev.is_some());
        assert!(ctx.next.is_some());
    }

    #[test]
    fn only_page_has_no_prev_next() {
        let config = test_config();
        let pages = vec![make_page("_site/index.html", "Home")];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert!(ctx.prev.is_none());
        assert!(ctx.next.is_none());
    }

    #[test]
    fn breadcrumbs_empty_for_root() {
        let config = test_config();
        let pages = vec![make_page("_site/index.html", "Home")];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert!(ctx.breadcrumbs.is_empty());
    }

    #[test]
    fn breadcrumbs_flat_includes_home() {
        let config = test_config();
        let pages = vec![
            make_page("_site/about.html", "About"),
            make_page("_site/index.html", "Home"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs.len(), 2);
        assert_eq!(ctx.breadcrumbs[0].title, "Home");
        assert!(ctx.breadcrumbs[0].href.is_some());
        assert_eq!(ctx.breadcrumbs[1].title, "About");
        assert!(ctx.breadcrumbs[1].href.is_none());
    }

    #[test]
    fn breadcrumbs_nested_has_dir_crumb() {
        let config = test_config();
        let pages = vec![
            make_page("_site/blog/post.html", "My Post"),
            make_page("_site/index.html", "Home"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs.len(), 3);
        assert_eq!(ctx.breadcrumbs[0].title, "Home");
        assert_eq!(ctx.breadcrumbs[1].title, "Blog");
        assert_eq!(ctx.breadcrumbs[2].title, "My Post");
        assert!(ctx.breadcrumbs[2].href.is_none());
    }

    #[test]
    fn sort_order() {
        let mut pages = vec![
            make_page("_site/index.html", "Home"),
            make_page("_site/about.html", "About"),
            make_page("_site/blog/post.html", "Post"),
        ];
        pages.sort_by(|a, b| a.output_path.cmp(&b.output_path));
        assert_eq!(pages[0].output_path, PathBuf::from("_site/about.html"));
        assert_eq!(pages[1].output_path, PathBuf::from("_site/blog/post.html"));
        assert_eq!(pages[2].output_path, PathBuf::from("_site/index.html"));
    }

    #[test]
    fn breadcrumbs_depth_3() {
        let config = test_config();
        let pages = vec![
            make_page("_site/a/b/c.html", "C Page"),
            make_page("_site/index.html", "Home"),
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
            make_page("_site/about.html", "About"),
            make_page("_site/index.html", "Welcome"),
        ];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs[0].title, "Welcome");
    }

    #[test]
    fn home_crumb_defaults_when_no_root_index() {
        let config = test_config();
        let pages = vec![make_page("_site/about.html", "About")];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs[0].title, "Home");
    }
}
