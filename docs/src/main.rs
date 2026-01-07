mod parsers;
use parsers::{JsonContentParser, TextContentParser, TomlContentParser};
use sherwood::plugins::PluginRegistry;

#[tokio::main]
async fn main() {
    let plugin_registry = PluginRegistry::new()
        .register("toml", TomlContentParser::new(), "toml")
        .register("json", JsonContentParser::new(), "json")
        .register("text", TextContentParser::new(), "txt")
        .map_extensions(&[("conf", "toml"), ("config", "toml"), ("schema", "json")]);

    let cli = sherwood::Sherwood::new().with_plugins(plugin_registry);

    if let Err(e) = cli.run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
