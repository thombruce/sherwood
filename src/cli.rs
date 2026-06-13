use std::borrow::Cow;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use clap::{Parser, Subcommand};

use crate::{BuildError, Page, PageContext, ParserRegistry, SiteConfig, build_site, serve};

/// A static asset written to the output directory after the site build.
///
/// `bytes` is `Cow` so callers can supply either compile-time `&'static [u8]`
/// (e.g. `include_bytes!(...)` or a bundled `&str`'s bytes) or an owned
/// `Vec<u8>` read from disk at runtime.
#[derive(Debug, Clone)]
pub struct Asset {
    /// Destination path relative to the output directory (e.g. `"style.css"`).
    pub dest: PathBuf,
    /// Asset bytes.
    pub bytes: Cow<'static, [u8]>,
}

impl Asset {
    pub fn new(dest: impl Into<PathBuf>, bytes: impl Into<Cow<'static, [u8]>>) -> Self {
        Self {
            dest: dest.into(),
            bytes: bytes.into(),
        }
    }
}

#[derive(Parser)]
#[command(name = "sherwood", version, about = "A static site generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the site from content/ to _site/
    Build {
        #[arg(long, default_value = "content")]
        content_dir: PathBuf,
        #[arg(long, default_value = "_site")]
        output_dir: PathBuf,
        /// Override a bundled asset with a file from disk. Format: `name=path`,
        /// where `name` matches an Asset's `dest`. May be repeated.
        #[arg(long, value_parser = parse_asset_override)]
        asset: Vec<(PathBuf, PathBuf)>,
    },
    /// Build then serve, with file watching and browser live reload.
    Serve {
        #[arg(long, default_value = "content")]
        content_dir: PathBuf,
        #[arg(long, default_value = "_site")]
        output_dir: PathBuf,
        #[arg(long, default_value_t = 4000)]
        port: u16,
        /// Override a bundled asset with a file from disk. Format: `name=path`.
        /// May be repeated. Re-applied on every rebuild.
        #[arg(long, value_parser = parse_asset_override)]
        asset: Vec<(PathBuf, PathBuf)>,
        /// Disable file watching and live reload. The dev server becomes a
        /// plain static-file server.
        #[arg(long)]
        no_watch: bool,
    },
}

fn parse_asset_override(raw: &str) -> Result<(PathBuf, PathBuf), String> {
    let (name, path) = raw
        .split_once('=')
        .ok_or_else(|| format!("expected `name=path`, got `{raw}`"))?;
    if name.is_empty() || path.is_empty() {
        return Err(format!("expected non-empty name and path, got `{raw}`"));
    }
    Ok((PathBuf::from(name), PathBuf::from(path)))
}

/// Run the standard Sherwood CLI (build + serve subcommands). Exits the
/// process with code 0 on success, 1 on failure. Use [`try_run_cli`] if you
/// want to handle errors yourself.
pub fn run_cli<F>(registry: ParserRegistry, renderer: F, assets: Vec<Asset>) -> ExitCode
where
    F: FnMut(&Page, &PageContext) -> Result<String, BuildError> + Send + 'static,
{
    match try_run_cli(registry, renderer, assets) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// Same as [`run_cli`] but returns the error instead of exiting.
pub fn try_run_cli<F>(
    registry: ParserRegistry,
    renderer: F,
    assets: Vec<Asset>,
) -> Result<(), CliError>
where
    F: FnMut(&Page, &PageContext) -> Result<String, BuildError> + Send + 'static,
{
    let cli = Cli::parse();
    match cli.command {
        Commands::Build {
            content_dir,
            output_dir,
            asset,
        } => {
            let assets = apply_overrides(assets, asset)?;
            let config = SiteConfig {
                content_dir,
                output_dir,
            };
            build_site(&config, &registry, renderer, |page| {
                println!(
                    "{} -> {}",
                    page.source_path.display(),
                    page.output_path.display()
                );
            })?;
            write_assets(&assets, &config)?;
            println!("Build complete.");
            Ok(())
        }
        Commands::Serve {
            content_dir,
            output_dir,
            port,
            asset,
            no_watch,
        } => {
            let assets = apply_overrides(assets, asset)?;
            let config = SiteConfig {
                content_dir: content_dir.clone(),
                output_dir: output_dir.clone(),
            };

            // Share the renderer + parsers + assets with the watcher's rebuild
            // closure.
            let renderer = Arc::new(Mutex::new(renderer));
            let registry = Arc::new(registry);
            let assets = Arc::new(assets);
            let config_for_rebuild = config.clone();
            let renderer_for_rebuild = renderer.clone();
            let registry_for_rebuild = registry.clone();
            let assets_for_rebuild = assets.clone();

            let rebuild = move || -> Result<(), BuildError> {
                let mut guard = renderer_for_rebuild
                    .lock()
                    .map_err(|_| BuildError::Render("renderer mutex poisoned".to_string()))?;
                let renderer_ref: &mut F = &mut guard;
                build_site(
                    &config_for_rebuild,
                    &registry_for_rebuild,
                    |p, c| renderer_ref(p, c),
                    |_| {},
                )?;
                write_assets(&assets_for_rebuild, &config_for_rebuild)
                    .map_err(|e| BuildError::Render(e.to_string()))?;
                Ok(())
            };

            let runtime = tokio::runtime::Runtime::new().map_err(CliError::Runtime)?;
            runtime.block_on(serve::serve_with_watch(
                content_dir,
                output_dir,
                port,
                rebuild,
                !no_watch,
            ))?;
            Ok(())
        }
    }
}

fn write_assets(assets: &[Asset], config: &SiteConfig) -> Result<(), CliError> {
    for a in assets {
        let dest = config.output_dir.join(&a.dest);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CliError::AssetWrite {
                path: dest.clone(),
                source: e,
            })?;
        }
        std::fs::write(&dest, &a.bytes).map_err(|e| CliError::AssetWrite {
            path: dest,
            source: e,
        })?;
    }
    Ok(())
}

fn apply_overrides(
    mut assets: Vec<Asset>,
    overrides: Vec<(PathBuf, PathBuf)>,
) -> Result<Vec<Asset>, CliError> {
    for (name, path) in overrides {
        let bytes = std::fs::read(&path).map_err(|e| CliError::AssetRead {
            path: path.clone(),
            source: e,
        })?;
        match assets.iter_mut().find(|a| a.dest == name) {
            Some(a) => a.bytes = Cow::Owned(bytes),
            None => assets.push(Asset {
                dest: name,
                bytes: Cow::Owned(bytes),
            }),
        }
    }
    Ok(assets)
}

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Build failed: {0}")]
    Build(#[from] BuildError),
    #[error("Serve failed: {0}")]
    Serve(#[from] serve::ServeError),
    #[error("Failed to start tokio runtime: {0}")]
    Runtime(std::io::Error),
    #[error("Failed to read asset {}: {source}", path.display())]
    AssetRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to write asset {}: {source}", path.display())]
    AssetWrite {
        path: PathBuf,
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_asset_override_ok() {
        let (name, path) = parse_asset_override("style.css=/tmp/x.css").unwrap();
        assert_eq!(name, PathBuf::from("style.css"));
        assert_eq!(path, PathBuf::from("/tmp/x.css"));
    }

    #[test]
    fn parse_asset_override_missing_equals() {
        assert!(parse_asset_override("style.css").is_err());
    }

    #[test]
    fn parse_asset_override_empty_parts() {
        assert!(parse_asset_override("=foo").is_err());
        assert!(parse_asset_override("foo=").is_err());
    }

    #[test]
    fn apply_overrides_replaces_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let style_path = tmp.path().join("new.css");
        std::fs::write(&style_path, b"body{}").unwrap();
        let assets = vec![Asset::new("style.css", Cow::Borrowed(&b"old"[..]))];
        let overrides = vec![(PathBuf::from("style.css"), style_path)];
        let out = apply_overrides(assets, overrides).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(&*out[0].bytes, b"body{}");
    }

    #[test]
    fn apply_overrides_appends_unknown() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("extra.txt");
        std::fs::write(&path, b"hi").unwrap();
        let assets = vec![];
        let overrides = vec![(PathBuf::from("extra.txt"), path)];
        let out = apply_overrides(assets, overrides).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].dest, PathBuf::from("extra.txt"));
    }

    #[test]
    fn apply_overrides_missing_file_errors() {
        let assets = vec![];
        let overrides = vec![(PathBuf::from("x"), PathBuf::from("/nonexistent/path/xyz"))];
        assert!(matches!(
            apply_overrides(assets, overrides),
            Err(CliError::AssetRead { .. })
        ));
    }
}
