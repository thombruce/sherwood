// Integration tests for the public library API — the path documented for
// downstream crates that depend on `sherwood`, define their own templates,
// and drive `build_site` with a custom render closure. These exercise only
// the re-exported surface (no CLI, no bundled template), so they double as a
// guard against accidental breakage of that contract.

use std::cell::{Cell, RefCell};
use std::fs;
use std::path::Path;

use sherwood::{BuildError, Page, PageContext, ParserRegistry, SiteConfig, build_site};
use tempfile::TempDir;

fn write(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

/// Lay down a small fixture site in a fresh temp dir and return it along with
/// a `SiteConfig` built through the public (non-exhaustive) builder API.
fn fixture() -> (TempDir, SiteConfig) {
    let tmp = TempDir::new().unwrap();
    let content = tmp.path().join("content");
    let output = tmp.path().join("out");

    write(
        &content.join("index.md"),
        "---\ntitle: Home\n---\n\n# Welcome\n",
    );
    write(
        &content.join("about.md"),
        "---\ntitle: About\nauthor: Thom\n---\n\nAbout body.\n",
    );
    write(
        &content.join("blog/index.md"),
        "---\ntitle: Blog\n---\n\nPosts.\n",
    );
    write(
        &content.join("blog/first.md"),
        "---\ntitle: First Post\n---\n\nIntro line.\n\n<!-- more -->\n\nRest of post.\n",
    );
    write(
        &content.join("blog/second.md"),
        "---\ntitle: Second Post\n---\n\nSecond body.\n",
    );

    let config = SiteConfig::new()
        .with_content_dir(content)
        .with_output_dir(output);
    (tmp, config)
}

#[test]
fn custom_renderer_output_is_written_at_pretty_urls() {
    let (_tmp, config) = fixture();
    let out = config.output_dir.clone();

    build_site(
        &config,
        &ParserRegistry::default(),
        |page: &Page, _ctx: &PageContext| {
            Ok(format!(
                "<article data-title=\"{}\">{}</article>",
                page.frontmatter.title, page.content_html
            ))
        },
        |_| {},
    )
    .unwrap();

    // Pretty-URL layout: every page becomes <dir>/index.html.
    let home = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(home.contains("data-title=\"Home\""));
    assert!(home.contains("<h1>Welcome</h1>"));

    let about = fs::read_to_string(out.join("about/index.html")).unwrap();
    assert!(about.contains("data-title=\"About\""));
    assert!(about.contains("<p>About body.</p>"));

    assert!(out.join("blog/index.html").exists());
    assert!(out.join("blog/first/index.html").exists());
    assert!(out.join("blog/second/index.html").exists());
}

#[test]
fn renderer_receives_nav_breadcrumbs_and_prev_next() {
    let (_tmp, config) = fixture();

    // Capture the context seen for the blog post so we can assert on it after
    // the build completes.
    let seen_nav: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let seen_crumbs: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let has_prev = Cell::new(false);
    let has_next = Cell::new(false);

    build_site(
        &config,
        &ParserRegistry::default(),
        |page: &Page, ctx: &PageContext| {
            if page.url == "/blog/first/" {
                *seen_nav.borrow_mut() = ctx.nav.iter().map(|n| n.title.clone()).collect();
                *seen_crumbs.borrow_mut() =
                    ctx.breadcrumbs.iter().map(|b| b.title.clone()).collect();
                has_prev.set(ctx.prev.is_some());
                has_next.set(ctx.next.is_some());
            }
            Ok(String::new())
        },
        |_| {},
    )
    .unwrap();

    // Default nav scoping: top-level pages + section indexes, leaves excluded.
    let nav = seen_nav.into_inner();
    assert!(nav.contains(&"Home".to_string()));
    assert!(nav.contains(&"About".to_string()));
    assert!(nav.contains(&"Blog".to_string()));
    assert!(
        !nav.contains(&"First Post".to_string()),
        "deep leaf must be excluded from default nav: {nav:?}"
    );

    // Breadcrumb trail: Home > Blog > First Post.
    assert_eq!(seen_crumbs.into_inner(), vec!["Home", "Blog", "First Post"]);

    // Prev/next is section-scoped: the first post has no prev (the Blog
    // section index chains in the top-level sequence, not among its posts)
    // and the second post as next.
    assert!(!has_prev.get());
    assert!(has_next.get());
}

#[test]
fn pages_under_drives_section_index() {
    let (_tmp, config) = fixture();
    let out = config.output_dir.clone();

    build_site(
        &config,
        &ParserRegistry::default(),
        |page: &Page, ctx: &PageContext| {
            // A section index lists its descendants via the public helper.
            if page.url == "/blog/" {
                let mut links: Vec<String> = ctx
                    .pages_under("/blog/")
                    .iter()
                    .filter(|p| p.url != "/blog/")
                    .map(|p| format!("<a href=\"{}\">{}</a>", p.url, p.frontmatter.title))
                    .collect();
                links.sort();
                return Ok(links.join("\n"));
            }
            Ok(String::new())
        },
        |_| {},
    )
    .unwrap();

    let blog = fs::read_to_string(out.join("blog/index.html")).unwrap();
    assert!(blog.contains("<a href=\"/blog/first/\">First Post</a>"));
    assert!(blog.contains("<a href=\"/blog/second/\">Second Post</a>"));
}

#[test]
fn renderer_can_read_custom_frontmatter_and_excerpt() {
    let (_tmp, config) = fixture();
    let out = config.output_dir.clone();

    build_site(
        &config,
        &ParserRegistry::default(),
        |page: &Page, _ctx: &PageContext| {
            let author = page
                .frontmatter
                .get_string("author")
                .unwrap_or_else(|| "anon".to_string());
            let excerpt = page.excerpt_html.clone().unwrap_or_default();
            Ok(format!("<meta data-author=\"{author}\">{excerpt}"))
        },
        |_| {},
    )
    .unwrap();

    let about = fs::read_to_string(out.join("about/index.html")).unwrap();
    assert!(about.contains("data-author=\"Thom\""));

    // The blog post defines an excerpt via the `<!-- more -->` delimiter.
    let post = fs::read_to_string(out.join("blog/first/index.html")).unwrap();
    assert!(post.contains("Intro line."));
    assert!(!post.contains("Rest of post."));
}

#[test]
fn progress_callback_runs_once_per_page() {
    let (_tmp, config) = fixture();
    let count = Cell::new(0usize);

    build_site(
        &config,
        &ParserRegistry::default(),
        |_page: &Page, _ctx: &PageContext| Ok(String::new()),
        |_page: &Page| count.set(count.get() + 1),
    )
    .unwrap();

    // Five source files in the fixture → five progress invocations.
    assert_eq!(count.get(), 5);
}

#[test]
fn renderer_error_propagates_as_build_error() {
    let (_tmp, config) = fixture();

    let result = build_site(
        &config,
        &ParserRegistry::default(),
        |_page: &Page, _ctx: &PageContext| Err(BuildError::Render("boom".to_string())),
        |_| {},
    );

    match result {
        Err(BuildError::Render(msg)) => assert_eq!(msg, "boom"),
        other => panic!("expected BuildError::Render, got {other:?}"),
    }
}

#[test]
fn malformed_frontmatter_surfaces_as_page_error() {
    let tmp = TempDir::new().unwrap();
    let content = tmp.path().join("content");
    write(&content.join("index.md"), "---\ntitle: Home\n---\n");
    // Missing the required `title` field.
    write(&content.join("bad.md"), "---\nfoo: bar\n---\n\nBody.\n");

    let config = SiteConfig::new()
        .with_content_dir(content)
        .with_output_dir(tmp.path().join("out"));

    let result = build_site(
        &config,
        &ParserRegistry::default(),
        |_p, _c| Ok(String::new()),
        |_| {},
    );

    let err = result.unwrap_err();
    assert!(matches!(err, BuildError::Page(_)));
    // The display chain carries the offending path and a line-numbered snippet.
    let msg = err.to_string();
    assert!(msg.contains("bad.md"), "path missing: {msg}");
    assert!(msg.contains("missing required field `title`"), "{msg}");
    assert!(msg.contains(" | "), "snippet missing: {msg}");
}

// --- Custom parser plugin -------------------------------------------------

use std::sync::Arc;

use sherwood::{ContentParser, FrontMatter, Parsed, ParserError, Pod};

/// A toy parser for a format the crate does not ship: `.shout` files, where the
/// first line is the title and the body is upper-cased into a `<p>`. Proves the
/// trait is open to brand-new formats, not just the built-ins.
struct ShoutParser;

impl ContentParser for ShoutParser {
    fn extensions(&self) -> &[&str] {
        &["shout"]
    }

    fn parse(&self, source: &str, _path: &Path) -> Result<Parsed, ParserError> {
        let mut lines = source.lines();
        let title = lines
            .next()
            .ok_or_else(|| ParserError::Message("empty file".to_string()))?
            .to_string();
        let body = lines.collect::<Vec<_>>().join(" ").to_uppercase();
        Ok(Parsed {
            frontmatter: FrontMatter {
                title,
                data: Pod::Null,
            },
            content_html: format!("<p>{body}</p>"),
            excerpt_html: None,
        })
    }
}

#[test]
fn user_registered_parser_handles_a_brand_new_extension() {
    let tmp = TempDir::new().unwrap();
    let content = tmp.path().join("content");
    let output = tmp.path().join("out");

    // A markdown page and a .shout page, side by side.
    write(&content.join("index.md"), "---\ntitle: Home\n---\n\n# Hi\n");
    write(&content.join("notes.shout"), "My Notes\nhello world\n");
    // An asset with no registered parser — copied through verbatim, not parsed.
    write(&content.join("logo.png"), "not really a png");

    let mut registry = ParserRegistry::default(); // markdown + text built in
    registry.register(Arc::new(ShoutParser));

    let config = SiteConfig::new()
        .with_content_dir(content)
        .with_output_dir(&output);

    build_site(
        &config,
        &registry,
        |page: &Page, _ctx: &PageContext| {
            Ok(format!(
                "<h1>{}</h1>{}",
                page.frontmatter.title, page.content_html
            ))
        },
        |_| {},
    )
    .unwrap();

    // Markdown still works.
    let home = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(home.contains("<h1>Home</h1>"));

    // The .shout file was parsed by the custom plugin and written at its own
    // pretty URL.
    let notes = fs::read_to_string(output.join("notes/index.html")).unwrap();
    assert!(notes.contains("<h1>My Notes</h1>"));
    assert!(notes.contains("<p>HELLO WORLD</p>"));

    // The unhandled asset produced no page — it was copied through verbatim.
    assert!(!output.join("logo/index.html").exists());
    assert_eq!(
        fs::read_to_string(output.join("logo.png")).unwrap(),
        "not really a png"
    );
}

#[test]
fn empty_registry_renders_no_pages() {
    let (_tmp, config) = fixture();
    let out = config.output_dir.clone();

    // No parsers registered → no file becomes a page; everything is treated
    // as a static asset and copied through verbatim.
    build_site(
        &config,
        &ParserRegistry::empty(),
        |_p: &Page, _c: &PageContext| Ok("x".to_string()),
        |_| {},
    )
    .unwrap();

    assert!(!out.join("index.html").exists());
    // The markdown source was passed through as-is, not rendered.
    let raw = fs::read_to_string(out.join("index.md")).unwrap();
    assert!(raw.contains("title: Home"));
}
