use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SiteConfig {
    pub site: SiteSection,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SiteSection {
    pub theme: Option<String>,
}
