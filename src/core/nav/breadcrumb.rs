use crate::core::config::SiteConfig;
use crate::core::content::page::Page;
use std::path::{Path, PathBuf};

use super::href_for;
use super::is_root_index;

#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub title: String,
    pub href: Option<String>,
}

pub(crate) fn breadcrumbs_for(
    page: &Page,
    all_pages: &[Page],
    config: &SiteConfig,
) -> Vec<Breadcrumb> {
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
    use crate::core::nav::compute_context;
    use crate::core::nav::test_support::{make_page, test_config};

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
        let pages = vec![make_page("about", "About"), make_page("index", "Home")];
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
    fn breadcrumbs_depth_3() {
        let config = test_config();
        let pages = vec![make_page("a/b/c", "C Page"), make_page("index", "Home")];
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
        let pages = vec![make_page("about", "About"), make_page("index", "Welcome")];
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
    fn breadcrumbs_dir_index_no_duplicate_leaf() {
        let config = test_config();
        let pages = vec![make_page("blog/index", "Blog"), make_page("index", "Home")];
        let ctx = compute_context(&pages[0], &pages, &config);
        assert_eq!(ctx.breadcrumbs.len(), 2);
        assert_eq!(ctx.breadcrumbs[0].title, "Home");
        assert_eq!(ctx.breadcrumbs[1].title, "Blog");
        assert!(ctx.breadcrumbs[1].href.is_none());
    }
}
