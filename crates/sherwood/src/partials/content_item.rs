use crate::presentation::templates::ListItemData;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "content_item.stpl")]
pub struct ContentItem {
    pub item: ListItemData,
}

impl ContentItem {
    pub fn new(data: ListItemData) -> Self {
        Self { item: data }
    }
}

impl From<ListItemData> for ContentItem {
    fn from(data: ListItemData) -> Self {
        Self { item: data }
    }
}
