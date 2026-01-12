pub mod common;
pub mod docs;
pub mod partials;
pub mod renderer;
pub mod sherwood;

// Re-exports for convenience
pub use common::{
    BreadcrumbData, BreadcrumbItem, ListData, ListItemData, NextPrevNavData, SidebarNavData,
    SidebarNavItem, TemplateError, TemplateInfo,
};
pub use docs::{DocsPageData, DocsTemplate};
pub use partials::{
    breadcrumb::{Breadcrumb, BreadcrumbGenerator},
    content_item::ContentItem,
    next_prev_nav::NextPrevNav,
    sidebar_nav::SidebarNav,
};
pub use renderer::{
    TemplateData, TemplateDataEnum, TemplateManager, copy_embedded_templates,
    get_available_templates,
};
pub use sherwood::{PageData, SherwoodTemplate};
