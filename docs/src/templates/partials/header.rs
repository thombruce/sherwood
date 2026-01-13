use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderData {
    pub logo: Option<LogoData>,
    pub navigation: Option<NavigationData>,
    pub search: Option<SearchData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoData {
    pub src: Option<String>,
    pub alt: Option<String>,
    pub text: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationData {
    pub items: Vec<NavigationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationItem {
    pub text: String,
    pub url: String,
    pub active: Option<bool>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchData {
    pub enabled: bool,
    pub placeholder: Option<String>,
    pub id: Option<String>,
}

#[derive(TemplateOnce, Debug)]
#[template(path = "partials/header.stpl")]
pub struct HeaderPartial {
    pub data: HeaderData,
}

impl HeaderPartial {
    pub fn new(data: HeaderData) -> Self {
        Self { data }
    }
}

impl Default for HeaderData {
    fn default() -> Self {
        Self {
            logo: Some(LogoData {
                src: None,
                alt: None,
                text: Some("Sherwood".to_string()),
                url: Some("/".to_string()),
            }),
            navigation: Some(NavigationData {
                items: vec![
                    NavigationItem {
                        text: "Docs".to_string(),
                        url: "/docs/".to_string(),
                        active: Some(false),
                        title: Some("Documentation".to_string()),
                    },
                    NavigationItem {
                        text: "Examples".to_string(),
                        url: "/examples/".to_string(),
                        active: Some(false),
                        title: Some("Examples".to_string()),
                    },
                    NavigationItem {
                        text: "GitHub".to_string(),
                        url: "https://github.com/thombruce/sherwood".to_string(),
                        active: Some(false),
                        title: Some("Sherwood on GitHub".to_string()),
                    },
                ],
            }),
            search: Some(SearchData {
                enabled: true,
                placeholder: Some("Search docs...".to_string()),
                id: Some("search-input".to_string()),
            }),
        }
    }
}
