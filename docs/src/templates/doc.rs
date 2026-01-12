use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "doc.stpl")]
pub struct DocTemplate {}
