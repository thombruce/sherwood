use sailfish::TemplateOnce;
use sherwood::templates::{FromTemplateData, TemplateData, TemplateDataEnum};

#[derive(TemplateOnce, Debug)]
#[template(path = "doc.stpl")]
pub struct DocTemplate {
    pub title: String,
    pub content: String,
}

impl FromTemplateData for DocTemplate {
    fn from(data: TemplateDataEnum) -> Self {
        Self {
            title: data.get_title().to_string(),
            content: data.get_content().to_string(),
        }
    }
}
