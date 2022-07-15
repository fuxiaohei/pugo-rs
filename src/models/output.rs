use crate::models;
use chrono::{DateTime, Utc};

pub struct Output {
    pub visit_url: String,
    pub output_files: Vec<String>,
    pub template_vars: models::GlobalVars,
    pub template_file: String,
    pub file_content: String,
    pub lastmod: DateTime<Utc>,
    pub sitemap_priority: f32,
}
