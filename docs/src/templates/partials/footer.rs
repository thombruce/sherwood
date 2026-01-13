use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FooterData {
    pub sections: Option<Vec<FooterSection>>,
    pub copyright: Option<CopyrightData>,
    pub social: Option<SocialData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FooterSection {
    pub title: Option<String>,
    pub links: Option<Vec<FooterLink>>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FooterLink {
    pub text: String,
    pub url: String,
    pub target: Option<String>,
    pub rel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyrightData {
    pub text: Option<String>,
    pub license: Option<LicenseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialData {
    pub items: Vec<SocialItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialItem {
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
    pub title: Option<String>,
    pub target: Option<String>,
}

#[derive(TemplateOnce, Debug)]
#[template(path = "partials/footer.stpl")]
pub struct FooterPartial {
    pub data: FooterData,
}

impl FooterPartial {
    pub fn new(data: FooterData) -> Self {
        Self { data }
    }
}

impl Default for FooterData {
    fn default() -> Self {
        Self {
            sections: Some(vec![
                FooterSection {
                    title: Some("Documentation".to_string()),
                    links: Some(vec![
                        FooterLink {
                            text: "Getting Started".to_string(),
                            url: "/docs/getting-started/".to_string(),
                            target: None,
                            rel: None,
                        },
                        FooterLink {
                            text: "Configuration".to_string(),
                            url: "/docs/configuration/".to_string(),
                            target: None,
                            rel: None,
                        },
                        FooterLink {
                            text: "Templates".to_string(),
                            url: "/docs/templates/".to_string(),
                            target: None,
                            rel: None,
                        },
                    ]),
                    content: None,
                },
                FooterSection {
                    title: Some("Resources".to_string()),
                    links: Some(vec![
                        FooterLink {
                            text: "Examples".to_string(),
                            url: "/examples/".to_string(),
                            target: None,
                            rel: None,
                        },
                        FooterLink {
                            text: "GitHub".to_string(),
                            url: "https://github.com/anomalyco/sherwood".to_string(),
                            target: Some("_blank".to_string()),
                            rel: Some("noopener noreferrer".to_string()),
                        },
                        FooterLink {
                            text: "Crates.io".to_string(),
                            url: "https://crates.io/crates/sherwood".to_string(),
                            target: Some("_blank".to_string()),
                            rel: Some("noopener noreferrer".to_string()),
                        },
                    ]),
                    content: None,
                },
                FooterSection {
                    title: Some("Community".to_string()),
                    links: Some(vec![
                        FooterLink {
                            text: "Issues".to_string(),
                            url: "https://github.com/anomalyco/sherwood/issues".to_string(),
                            target: Some("_blank".to_string()),
                            rel: Some("noopener noreferrer".to_string()),
                        },
                        FooterLink {
                            text: "Discussions".to_string(),
                            url: "https://github.com/anomalyco/sherwood/discussions".to_string(),
                            target: Some("_blank".to_string()),
                            rel: Some("noopener noreferrer".to_string()),
                        },
                    ]),
                    content: None,
                },
            ]),
            copyright: Some(CopyrightData {
                text: Some("© 2024 Sherwood Static Site Generator".to_string()),
                license: Some(LicenseData {
                    name: "MIT License".to_string(),
                    url: Some("https://opensource.org/licenses/MIT".to_string()),
                }),
            }),
            social: Some(SocialData {
                items: vec![
                    SocialItem {
                        name: "GitHub".to_string(),
                        url: "https://github.com/anomalyco/sherwood".to_string(),
                        icon: Some("<svg width=\"16\" height=\"16\" viewBox=\"0 0 16 16\" fill=\"currentColor\"><path d=\"M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z\"/></svg>".to_string()),
                        title: Some("Sherwood on GitHub".to_string()),
                        target: Some("_blank".to_string()),
                    },
                ],
            }),
        }
    }
}
