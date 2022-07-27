use crate::models;
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    pub name: String,
    pub url: String,
    pub page_format: String,
    pub posts_index: Vec<usize>,
}

fn build_tag_link(tag: &str, format: &str) -> String {
    let link = format.to_string();
    link.replace(":tag", tag)
}

impl Tag {
    pub fn parse(posts: &[models::Post], url_config: &models::UrlConfig) -> Vec<Tag> {
        let mut tags = std::collections::HashMap::new();
        for (index, p) in posts.iter().enumerate() {
            for t in p.meta.tags.as_ref().unwrap() {
                let tag = tags.entry(t.clone()).or_insert(Tag {
                    name: t.clone(),
                    url: build_tag_link(t.as_str(), url_config.tag_link_format.as_str()),
                    page_format: build_tag_link(t.as_str(), url_config.tag_page_format.as_str()),
                    posts_index: vec![],
                });
                tag.posts_index.push(index);
            }
        }
        let mut values: Vec<Tag> = tags.into_values().collect();
        values.sort_by(|a, b| b.posts_index.len().cmp(&a.posts_index.len()));
        values
    }
}

#[cfg(test)]
mod post_tags_test {
    use super::*;

    #[test]
    fn test_tags() {
        let posts = vec![
            models::Post {
                meta: models::PostMetadata {
                    tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
                    ..models::PostMetadata::default()
                },
                ..Default::default()
            },
            models::Post {
                meta: models::PostMetadata {
                    tags: Some(vec!["tag2".to_string(), "tag3".to_string()]),
                    ..models::PostMetadata::default()
                },
                ..Default::default()
            },
            models::Post {
                meta: models::PostMetadata {
                    tags: Some(vec!["tag2".to_string(), "tag1".to_string()]),
                    ..models::PostMetadata::default()
                },
                ..Default::default()
            },
        ];
        let url_config = models::UrlConfig::new();
        let tags = Tag::parse(&posts, &url_config);
        assert_eq!(tags.len(), 3); // tag1, tag2, tag3
        assert_eq!(tags[0].name, "tag2"); // tag2 has the most posts
        assert_eq!(tags[0].posts_index.len(), 3); // index of post1 and post2 and post3

        assert_eq!(tags[1].name, "tag1");
        assert_eq!(tags[1].posts_index.len(), 2); // index of post1, post3
    }
}
