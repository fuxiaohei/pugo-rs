use crate::models;

pub struct Output {
    pub visit_url: String,
    pub output_file: String,
    pub template_vars: models::GlobalVars,
    pub template_file: String,
}
