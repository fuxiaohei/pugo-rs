use crate::models;
use log::{debug, info};

pub struct Site {
    pub config: models::Config,
    pub posts: Vec<models::Post>,
    pub pages: Vec<models::Post>,
    pub tags: Vec<models::Tag>,
}

impl Site {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. read config
        let config = models::Config::from_file(path)?;
        debug!("Loaded config: {}", path);

        // 2. load sources
        let posts = models::Post::list_from_dir(&config.get_posts_dir())?;
        info!("Loaded posts: {}", posts.len());
        let pages = models::Post::list_from_dir(&config.get_pages_dir())?;
        info!("Loaded pages: {}", pages.len());

        let mut site = Site {
            config,
            posts,
            pages,
            tags: vec![],
        };
        site.parse_source()?;
        Ok(site)
    }

    fn parse_source(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. parse tags
        self.tags = models::Tag::parse(&self.posts, &self.config.url);
        debug!("Loaded tags: {}", self.tags.len());

        // 2. parse posts
        let slug_format = self.config.get_slug_link();
        for p in &mut self.posts {
            p.set_slug_url(&slug_format);
            if p.meta.author.is_none() {
                p.meta.author = Some(self.config.site.author.clone());
            }
            if p.meta.language.is_none() {
                p.meta.language = Some(self.config.site.language.clone());
            }
            p.author = Some(self.config.get_author(&p.meta.author.as_ref().unwrap()));
            p.brief_html = markdown_to_html(&p.brief_markdown);
            p.content_html = markdown_to_html(&p.content_markdown);
        }

        // 3. parse pages
        for p in &mut self.pages {
            p.slug_url = self.config.build_root_url(&p.meta.slug);
            if p.meta.author.is_none() {
                p.meta.author = Some(self.config.site.author.clone());
            }
            if p.meta.language.is_none() {
                p.meta.language = Some(self.config.site.language.clone());
            }
            // page's brief is empty
            // p.brief_html = markdown_to_html(&p.brief_markdown);
            p.content_html = markdown_to_html(&p.content_markdown);
            p.author = Some(self.config.get_author(&p.meta.author.as_ref().unwrap()));

            // use page.hbs instead of post.hbs as default post
            if p.meta.template.as_ref().unwrap() == "post.hbs" {
                p.meta.template = Some("page.hbs".to_string());
            }
        }

        Ok(())
    }

    pub fn build(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

pub fn markdown_to_html(content: &str) -> String {
    use pulldown_cmark as cmark;
    let mut buf = String::new();
    let options = cmark::Options::ENABLE_FOOTNOTES
        | cmark::Options::ENABLE_TABLES
        | cmark::Options::ENABLE_STRIKETHROUGH
        | cmark::Options::ENABLE_TASKLISTS;
    let parser = cmark::Parser::new_ext(content, options);
    cmark::html::push_html(&mut buf, parser);
    buf
}
