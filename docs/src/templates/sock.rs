use sailfish::TemplateOnce;
use sherwood::templates::{FromTemplateData, TemplateData, TemplateDataEnum};

#[derive(TemplateOnce, Debug)]
#[template(path = "sock.stpl")]
pub struct SockTemplate {
    pub title: String,
    pub content: String,
}

impl FromTemplateData for SockTemplate {
    fn from(data: TemplateDataEnum) -> Self {
        Self {
            title: data.get_title().to_string(),
            content: data.get_content().to_string(),
        }
    }
}
