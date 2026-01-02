use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SiteConfig {
    pub site: SiteSection,
    pub templates: Option<TemplateSection>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SiteSection {}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateSection {
    pub page_template: Option<String>,
}
