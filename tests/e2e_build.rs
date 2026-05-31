// End-to-end test for the `sherwood build` subcommand. Exercises the full
// binary: clap CLI parsing, the build pipeline, the bundled default
// template, and the asset writer. Run via `cargo test --test e2e_build`.

use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn write(path: &std::path::Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

#[test]
fn build_full_site_against_fixture() {
    let bin = env!("CARGO_BIN_EXE_sherwood");
    let tmp = TempDir::new().unwrap();
    let content = tmp.path().join("content");
    let output = tmp.path().join("out");

    write(
        &content.join("index.md"),
        "---\ntitle: Home\n---\n\n# Welcome\n",
    );
    write(
        &content.join("about.md"),
        "---\ntitle: About\ndescription: About this site.\n---\n\nAbout body.\n",
    );
    write(
        &content.join("blog/index.md"),
        "---\ntitle: Blog\n---\n\nPost list.\n",
    );
    write(
        &content.join("blog/first.md"),
        "---\ntitle: First Post\ndate: 2026-05-30\n---\n\nIntro.\n\n<!-- more -->\n\nRest.\n",
    );

    let status = Command::new(bin)
        .args([
            "build",
            "--content-dir",
            content.to_str().unwrap(),
            "--output-dir",
            output.to_str().unwrap(),
        ])
        .status()
        .expect("failed to launch sherwood binary");
    assert!(status.success(), "sherwood build exited non-zero");

    // Pretty-URL output structure
    assert!(output.join("index.html").exists(), "/ missing");
    assert!(output.join("about/index.html").exists(), "/about/ missing");
    assert!(output.join("blog/index.html").exists(), "/blog/ missing");
    assert!(
        output.join("blog/first/index.html").exists(),
        "/blog/first/ missing"
    );

    // Bundled stylesheet written
    let css = fs::read_to_string(output.join("style.css")).unwrap();
    assert!(!css.is_empty(), "style.css empty");

    // Home renders body + bundled nav linking the section index
    let home = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(home.contains("<title>Home</title>"));
    assert!(home.contains("<h1>Welcome</h1>"));
    assert!(home.contains("<link rel=\"stylesheet\" href=\"/style.css\">"));
    assert!(home.contains("href=\"/blog/\""), "nav should link /blog/");
    // Blog post is a leaf — nav scoping should hide it
    assert!(
        !home.contains("href=\"/blog/first/\""),
        "deep leaf must be excluded from default nav, got:\n{home}"
    );

    // About page produces wrapped output, current-page nav marker, body
    let about = fs::read_to_string(output.join("about/index.html")).unwrap();
    assert!(about.contains("<title>About</title>"));
    assert!(about.contains("<p>About body.</p>"));
    assert!(
        about.contains("href=\"/about/\" aria-current=\"page\""),
        "current page should be marked"
    );

    // Blog post: breadcrumb includes Home + Blog + leaf title
    let post = fs::read_to_string(output.join("blog/first/index.html")).unwrap();
    assert!(post.contains("<title>First Post</title>"));
    assert!(post.contains("href=\"/blog/\""));
}

#[test]
fn build_reports_frontmatter_error_with_snippet() {
    let bin = env!("CARGO_BIN_EXE_sherwood");
    let tmp = TempDir::new().unwrap();
    let content = tmp.path().join("content");
    let output = tmp.path().join("out");

    write(&content.join("index.md"), "---\ntitle: Home\n---\n");
    write(
        &content.join("bad.md"),
        "---\nfoo: bar\nauthor: thom\n---\n\nBody.\n",
    );

    let result = Command::new(bin)
        .args([
            "build",
            "--content-dir",
            content.to_str().unwrap(),
            "--output-dir",
            output.to_str().unwrap(),
        ])
        .output()
        .expect("failed to launch sherwood binary");

    assert!(!result.status.success(), "build should fail on bad frontmatter");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("missing required field `title`"), "stderr:\n{stderr}");
    assert!(stderr.contains("foo: bar"), "snippet missing in stderr:\n{stderr}");
    assert!(stderr.contains(" | "), "line-numbered indent missing:\n{stderr}");
}
