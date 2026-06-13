# sherwood-site

The Sherwood marketing/documentation site — **built with Sherwood**, using the
`sherwood` crate as a library. This is the project's dogfood: if building this
site is awkward, that's API feedback.

It depends on `sherwood` with `default-features = false, features = ["cli"]`
(the core library + the `run_cli` helper, but *not* the bundled template), and
ships its own Sailfish template (`templates/page.stpl`) and stylesheet
(`assets/style.css`).

## Build / serve

From the repository root:

```bash
cargo run -p sherwood-site -- build --content-dir site/content --output-dir site/_site
cargo run -p sherwood-site -- serve --content-dir site/content --output-dir site/_site
```

Output (`site/_site/`) is gitignored. `sherwood-site` is `publish = false` and is
excluded from the published `sherwood` crate.
