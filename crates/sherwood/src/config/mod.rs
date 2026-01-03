use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SiteConfig {
    pub site: SiteSection,
    pub templates: Option<TemplateSection>,
    pub css: Option<CssSection>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SiteSection {}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateSection {
    pub page_template: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CssSection {
    pub minify: Option<bool>,
    pub targets: Option<CssTargets>,
    pub source_maps: Option<bool>,
    pub remove_unused: Option<bool>,
    pub nesting: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CssTargets {
    pub chrome: Option<String>,
    pub firefox: Option<String>,
    pub safari: Option<String>,
    pub edge: Option<String>,
    pub browserslist: Option<String>,
}
