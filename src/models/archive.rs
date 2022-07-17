use crate::models;
use chrono::Datelike;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Archive {
    pub year: String,
    pub posts_index: Vec<usize>,
}

impl Archive {
    pub fn parse(posts: &[models::Post]) -> Vec<Archive> {
        let mut archives = std::collections::HashMap::new();
        for (index, p) in posts.iter().enumerate() {
            let year = p.datetime.unwrap().year();
            let archive = archives.entry(year.to_string()).or_insert(Archive {
                year: year.to_string(),
                posts_index: vec![],
            });
            archive.posts_index.push(index);
        }
        let mut values: Vec<Archive> = archives.into_values().collect();
        values.sort_by(|a, b| b.year.cmp(&a.year));
        values
    }
}
