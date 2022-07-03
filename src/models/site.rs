use crate::models;
use log::{debug, info};

pub struct Site {
    pub config: models::Config,
    pub posts: Vec<models::Post>,
    pub pages: Vec<models::Post>,
    pub tags: Vec<models::Tag>,

    template_vars: models::TemplateVars,
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
            template_vars: models::TemplateVars::default(),
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

        // 4. after all parsed, generate global template
        self.template_vars = models::TemplateVars::new(self);

        Ok(())
    }

    pub fn build(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut outputs = vec![];
        
        // 1. build posts
        outputs.extend(self.build_posts()?);

        // 2. build pages
        self.build_pages()?;

        // 3. build tags
        self.build_tags()?;

        debug!("Built outputs: {}", outputs.len());

        Ok(())
    }

    fn build_posts(&self) -> Result<Vec<models::Output>, Box<dyn std::error::Error>> {
        let mut outputs = vec![];

        // build each post
        for p in &self.posts {
            let output_file = self.config.build_dist_html_filepath(&p.slug_url, true);
            let mut template_vars = self.template_vars.get_global();
            template_vars.post = Some(self.template_vars.build_postvars(p));
            outputs.push(models::Output {
                visit_url: self.config.build_root_url(&p.slug_url),
                output_file,
                template_vars,
                template_file: p.meta.template.as_ref().unwrap().clone(),
            });
        }

        // build pagination
        let pagination = models::Pagination::new(self.posts.len(), self.config.url.per_page_size);
        for i in 0..pagination.total_pages {
            let current_page = pagination.build_each_page(i + 1, &self.config.url.post_page_format);
            let output_file = self
                .config
                .build_dist_html_filepath(&current_page.current_url(), true);
            let mut template_vars = self.template_vars.get_global();
            template_vars.pagination = Some(current_page.build_template_vars());
            let posts = &self.posts[current_page.start..current_page.end];
            let mut posts_vars = Vec::new();
            for p in posts {
                let post_vars = self.template_vars.build_postvars(p);
                posts_vars.push(post_vars);
            }
            template_vars.posts = Some(posts_vars);
            outputs.push(models::Output {
                visit_url: self.config.build_root_url(&current_page.current_url()),
                output_file,
                template_vars,
                template_file: String::from("posts.hbs"),
            });
        }
        Ok(outputs)
    }

    fn build_tags(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn build_pages(&self) -> Result<(), Box<dyn std::error::Error>> {
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
