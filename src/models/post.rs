use crate::models;
use chrono::Datelike;
use log::debug;

pub fn parse_time(time_str: &str) -> Result<chrono::NaiveDateTime, Box<dyn std::error::Error>> {
    let mut time_text = time_str.trim().to_string();
    if time_text.len() == 10 {
        time_text = format!("{time_str} 00:00:00");
    } else if time_text.len() == 16 {
        time_text = format!("{time_str}:00");
    }
    if time_text.len() != 19 {
        return Err(format!("time string is not valid: {}", time_str).into());
    }
    Ok(chrono::NaiveDateTime::parse_from_str(
        &time_text,
        "%Y-%m-%d %H:%M:%S",
    )?)
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PostMetadata {
    pub title: String,
    pub slug: String,
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

#[derive(Debug, Default)]
pub struct Post {
    pub meta: PostMetadata,
    pub brief_markdown: String,
    pub content_markdown: String,
    pub brief_html: String,
    pub content_html: String,
    pub datetime: Option<chrono::NaiveDateTime>,
    pub updated_datetime: Option<chrono::NaiveDateTime>,
    pub author: Option<models::Author>,

    pub local_file: String,
    pub slug_url: String,
}

impl Post {
    fn default() -> Post {
        Post {
            meta: PostMetadata {
                comments: Some(true),
                ..PostMetadata::default()
            },
            ..Default::default()
        }
    }

    pub fn parse_meta(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // fill default values
        if self.meta.slug.starts_with('/') {
            self.meta.slug = self.meta.slug.strip_prefix('/').unwrap().to_string();
        }
        if self.meta.template.is_none() {
            self.meta.template = Some("post.hbs".to_string());
        }
        if self.meta.comments.is_none() {
            self.meta.comments = Some(true);
        }
        if self.meta.updated.is_none() {
            self.meta.updated = Some(self.meta.date.clone());
        }
        if self.meta.tags.is_none() {
            self.meta.tags = Some(vec![]); // fill empty slice to make sure other functions working
        }

        // parse time
        self.datetime = Some(parse_time(self.meta.date.as_str())?);
        self.updated_datetime = Some(parse_time(self.meta.updated.as_ref().unwrap().as_str())?);

        // parse brief
        let mut seperator_index = self.content_markdown.find("<!-- more -->").unwrap_or(0);
        if seperator_index < 1 {
            seperator_index = self.content_markdown.find("<!--more-->").unwrap_or(0);
        }
        if seperator_index > 0 {
            self.brief_markdown = self
                .content_markdown
                .split_at(seperator_index)
                .0
                .trim()
                .to_string();
        } else {
            self.brief_markdown = self.content_markdown.clone();
        }
        Ok(())
    }

    pub fn from_str(content: &str) -> Result<Post, Box<dyn std::error::Error>> {
        let mut metadata_string = String::from("");
        let mut metadata_format = "yaml";
        let mut content_string = String::from("");
        let mut metadata_section_flag = false;
        let mut content_lines = content.trim().lines();
        for line in content_lines.by_ref() {
            // start with --- or ```toml , enter as metadata
            if !metadata_section_flag {
                if line == "---" {
                    metadata_section_flag = true;
                    continue;
                } else if line == "```toml" {
                    metadata_format = "toml";
                    metadata_section_flag = true;
                    continue;
                }
            }

            if metadata_section_flag {
                // find --- again, metadata is end
                if line == "---" && metadata_format == "yaml" {
                    metadata_section_flag = false;
                    metadata_string.push('\n');
                    continue;
                }
                if line == "```" && metadata_format == "toml" {
                    metadata_section_flag = false;
                    metadata_string.push('\n');
                    continue;
                }
                metadata_string.push_str(line);
                metadata_string.push('\n');
                continue;
            }
            // metadata is end, left lines are content
            content_string.push_str(line);
            content_string.push('\n');
        }
        let mut post = Post::default();
        if metadata_format == "yaml" {
            post.meta = serde_yaml::from_str(metadata_string.as_str())?;
        } else if metadata_format == "toml" {
            post.meta = toml::from_str(metadata_string.as_str())?;
        }
        post.content_markdown = content_string.trim().to_string();
        post.parse_meta()?;
        Ok(post)
    }

    pub fn from_file(file_path: &str) -> Result<Post, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut post = Post::from_str(&content)?;
        post.local_file = file_path.to_string();
        Ok(post)
    }

    pub fn list_from_dir(dir_path: &str) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let mut posts = Vec::new();
        for entry in walkdir::WalkDir::new(dir_path) {
            let entry = entry.unwrap();
            let post_file_path = entry.path();
            let post_file_path_str = post_file_path.to_str().unwrap();
            if post_file_path.is_file() && post_file_path_str.ends_with(".md") {
                let post = match Post::from_file(post_file_path_str) {
                    Ok(post) => post,
                    Err(e) => {
                        return Err(e);
                    }
                };
                posts.push(post);
                debug!("Loaded source: {}", post_file_path_str);
            }
        }
        // sort by date
        posts.sort_by(|a, b| b.datetime.unwrap().cmp(&a.datetime.unwrap()));
        Ok(posts)
    }

    pub fn to_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut post_string = String::from("");
        let metadata_string = serde_yaml::to_string(&self.meta).unwrap();
        post_string.push_str(metadata_string.as_str());
        post_string.push_str("---\n\n");
        post_string.push_str(&self.content_markdown);
        post_string.push('\n');
        std::fs::write(file_path, post_string).unwrap();
        Ok(())
    }

    pub fn set_slug_url(&mut self, permalink: &str) {
        let datetime = self.datetime.unwrap();
        self.slug_url = permalink
            .replace(":year", datetime.year().to_string().as_str())
            .replace(":month", format!("{:0>2}", datetime.month()).as_str())
            .replace(":day", format!("{:0>2}", datetime.day()).as_str())
            .replace(":slug", self.meta.slug.as_str());
    }
}

#[cfg(test)]
mod parse_time_tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_parse_time_seconds() {
        let time_str = "2020-01-02 03:04:05";
        let datetime = parse_time(time_str).unwrap();
        assert_eq!(datetime.year(), 2020);
        assert_eq!(datetime.month(), 1);
        assert_eq!(datetime.day(), 2);
        assert_eq!(datetime.hour(), 3);
        assert_eq!(datetime.minute(), 4);
        assert_eq!(datetime.second(), 5);
    }

    #[test]
    fn test_parse_time_minutes() {
        let time_str = "2020-01-02 03:04";
        let datetime = parse_time(time_str).unwrap();
        assert_eq!(datetime.year(), 2020);
        assert_eq!(datetime.month(), 1);
        assert_eq!(datetime.day(), 2);
        assert_eq!(datetime.hour(), 3);
        assert_eq!(datetime.minute(), 4);
        assert_eq!(datetime.second(), 0);
    }

    #[test]
    fn test_parse_time_days() {
        let time_str = "2020-01-02";
        let datetime = parse_time(time_str).unwrap();
        assert_eq!(datetime.year(), 2020);
        assert_eq!(datetime.month(), 1);
        assert_eq!(datetime.day(), 2);
        assert_eq!(datetime.hour(), 0);
        assert_eq!(datetime.minute(), 0);
        assert_eq!(datetime.second(), 0);
    }
}

#[cfg(test)]
mod post_tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    fn create_test_post_content() -> String {
        let post_content = "---
title: Deploy blog in Cloudflare Workers
author: admin
date: 2022-05-25 15:55:25
tags:
- cloudflare
- blog
- workers
slug: blog-cf-worker
---
Workers Sites leverages the power of Workers and Workers KV by allowing developers to upload their sites directly to the edge, and closer to the end users. <!-- more --> Born on the edge, Workers Sites is what we think modern development on the web should look like, natively secure, fast, and massively scalable. Less of your time is spent on configuration, and more of your time is spent on your code, and content itself.";
        post_content.to_string()
    }

    fn create_test_post_content_toml() -> String {
        let post_content = "```toml
title = \"Deploy blog in Cloudflare Workers 2\"
author = \"admin\"
date = \"2022-06-26 18:30:30\"
tags = [ \"cloudflare\", \"blog\", \"workers\" ]
slug = \"blog-cf-worker\"
```
Workers Sites leverages the power of Workers and Workers KV by allowing developers to upload their sites directly to the edge, and closer to the end users. <!--more--> Born on the edge, Workers Sites is what we think modern development on the web should look like, natively secure, fast, and massively scalable. Less of your time is spent on configuration, and more of your time is spent on your code, and content itself.";
        post_content.to_string()
    }

    #[test]
    fn test_parse_post() {
        let mut post = Post::from_str(&create_test_post_content()).unwrap();
        assert_eq!(post.meta.title, "Deploy blog in Cloudflare Workers");
        assert_eq!(post.meta.slug, "blog-cf-worker");
        assert_eq!(post.meta.date, "2022-05-25 15:55:25");
        assert_eq!(post.meta.updated, Some("2022-05-25 15:55:25".to_string())); // updated is same to date if not set
        assert_eq!(
            post.meta.tags.as_ref().unwrap().join(","),
            "cloudflare,blog,workers"
        );
        assert_eq!(post.meta.template, Some("post.hbs".to_string())); // template is post.hbs as default if not set
        assert_eq!(post.meta.language, None);
        assert_eq!(post.meta.comments, Some(true));
        assert_eq!(post.meta.author, Some("admin".to_string()));
        assert_eq!(post.content_markdown.len(), 423);

        // brief should be parsed if seperator is found
        assert_eq!(post.brief_markdown.len(), 155);

        // datetime is parsed
        assert!(post.datetime.unwrap().year() == 2022);
        assert!(post.datetime.unwrap().month() == 5);
        assert!(post.updated_datetime.unwrap().hour() == 15);
        assert!(post.updated_datetime.unwrap().minute() == 55);
        assert!(post.updated_datetime.unwrap().second() == 25);

        // slug format
        post.set_slug_url("aaa/:year/:month/:day/:slug");
        assert_eq!(post.slug_url, "aaa/2022/05/25/blog-cf-worker");
    }

    #[test]
    fn test_parse_post_file() {
        // use toml format to test
        std::fs::write("test_post.md", &create_test_post_content_toml()).unwrap();
        let post = Post::from_file("test_post.md").unwrap();
        assert_eq!(post.meta.title, "Deploy blog in Cloudflare Workers 2");
        assert_eq!(post.meta.date, "2022-06-26 18:30:30");
        assert_eq!(post.content_markdown.len(), 421);
        assert_eq!(post.brief_markdown.len(), 155);
        assert!(post.updated_datetime.unwrap().hour() == 18);
        assert!(post.updated_datetime.unwrap().minute() == 30);
        assert!(post.updated_datetime.unwrap().second() == 30);
        std::fs::remove_file("test_post.md").unwrap();
    }

    #[test]
    fn test_parse_post_dir() {
        std::fs::create_dir_all("test_post_dir").unwrap();
        std::fs::write("test_post_dir/post_one_1.md", &create_test_post_content()).unwrap();
        std::fs::write(
            "test_post_dir/post_one_2.md",
            &create_test_post_content_toml(),
        )
        .unwrap();
        let posts = Post::list_from_dir("test_post_dir").unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].meta.title, "Deploy blog in Cloudflare Workers 2");
        assert_eq!(posts[1].meta.title, "Deploy blog in Cloudflare Workers");

        std::fs::remove_dir_all("test_post_dir").unwrap();
    }
}
