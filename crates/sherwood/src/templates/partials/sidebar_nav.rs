use crate::templates::SidebarNavData;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "partials/sidebar_nav.stpl")]
pub struct SidebarNav {
    pub sidebar_nav: SidebarNavData,
}

impl SidebarNav {
    pub fn new(data: SidebarNavData) -> Self {
        Self { sidebar_nav: data }
    }
}
