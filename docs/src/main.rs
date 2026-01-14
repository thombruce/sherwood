mod parsers;
mod templates;
use crate::templates::docs::DocsTemplate;
use parsers::{JsonContentParser, TextContentParser, TomlContentParser};
use sherwood::plugins::PluginRegistry;
use sherwood::templates::SherwoodTemplate;
use sherwood::{TemplateRegistry, register_template};

fn create_template_registry() -> TemplateRegistry {
    let mut registry = TemplateRegistry::new();

    // Register built-in templates to maintain backward compatibility
    register_template!(registry, "sherwood.stpl", SherwoodTemplate).unwrap();

    // Register custom template
    register_template!(registry, "docs.stpl", DocsTemplate).unwrap();

    registry
}

#[tokio::main]
async fn main() {
    let plugin_registry = PluginRegistry::new()
        .register("toml", TomlContentParser::new(), "toml")
        .register("json", JsonContentParser::new(), "json")
        .register("text", TextContentParser::new(), "txt")
        .map_extensions(&[("conf", "toml"), ("config", "toml"), ("schema", "json")]);

    let template_registry = create_template_registry();

    let cli = sherwood::Sherwood::new()
        .with_plugins(plugin_registry)
        .with_templates(template_registry);

    if let Err(e) = cli.run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
