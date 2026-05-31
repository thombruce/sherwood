use std::process::ExitCode;

use sherwood::{Asset, DEFAULT_STYLE, render_page, run_cli};

fn main() -> ExitCode {
    run_cli(
        render_page,
        vec![Asset::new("style.css", DEFAULT_STYLE.as_bytes())],
    )
}
