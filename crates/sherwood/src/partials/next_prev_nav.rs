use crate::presentation::templates::NextPrevNavData;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "partials/next_prev_nav.stpl")]
pub struct NextPrevNav {
    pub nav: NextPrevNavData,
}

impl NextPrevNav {
    pub fn new(data: NextPrevNavData) -> Self {
        Self { nav: data }
    }
}
