use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SiteConfig {
    pub site: SiteSection,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SiteSection {
    pub theme: Option<String>,
    pub navigation: Option<Navigation>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Navigation {
    pub items: Vec<NavigationItem>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NavigationItem {
    pub title: String,
    pub url: String,
}
