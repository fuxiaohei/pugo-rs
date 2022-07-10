use crate::models;
use log::{debug, error, info};

pub struct Site<'a> {
    pub config: models::Config,
    pub posts: Vec<models::Post>,
    pub pages: Vec<models::Post>,
    pub tags: Vec<models::Tag>,
    pub theme: models::Theme<'a>,

    template_vars: models::TemplateVars,
}

impl Site<'_> {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. read config
        let config = models::Config::from_file(path)?;
        debug!("Loaded config: {}", path);

        // 2. load sources
        let posts = models::Post::list_from_dir(&config.get_posts_dir())?;
        info!("Loaded posts: {}", posts.len());
        let pages = models::Post::list_from_dir(&config.get_pages_dir())?;
        info!("Loaded pages: {}", pages.len());

        // 3. parse theme
        let theme_dir = config.get_theme_dir();
        let theme = models::Theme::parse(&theme_dir)?;
        info!("Loaded theme: {}", &theme_dir);

        let mut site = Site {
            config,
            posts,
            pages,
            tags: vec![],
            template_vars: models::TemplateVars::default(),
            theme,
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
            p.author = Some(self.config.get_author(p.meta.author.as_ref().unwrap()));
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
            p.author = Some(self.config.get_author(p.meta.author.as_ref().unwrap()));

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
        outputs.extend(self.build_pages()?);

        // 3. build tags
        outputs.extend(self.build_tags()?);

        // 4. build index
        outputs.extend(self.build_index()?);

        // 5. generate files
        let generated_count = self.generate_files(&outputs)?;

        // 6. copy static files
        self.copy_assets();

        debug!("Generate files: {}", generated_count);

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
                output_files: vec![output_file],
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
                output_files: vec![output_file],
                template_vars,
                template_file: String::from("posts.hbs"),
            });
        }
        Ok(outputs)
    }

    fn build_tags(&self) -> Result<Vec<models::Output>, Box<dyn std::error::Error>> {
        let mut outputs = vec![];

        for tag in &self.tags {
            let pagination =
                models::Pagination::new(tag.posts_index.len(), self.config.url.per_page_size);
            for i in 0..pagination.total_pages {
                let current_page = pagination.build_each_page(i + 1, &tag.page_format);
                let output_file = self
                    .config
                    .build_dist_html_filepath(&current_page.current_url(), true);

                // create template vars
                let mut template_vars = self.template_vars.get_global();
                template_vars.pagination = Some(current_page.build_template_vars());
                template_vars.current_tag = self.template_vars.get_tag(&tag.name);

                // set post vars list
                let mut posts_vars = Vec::new();
                for index in &tag.posts_index {
                    let post_vars = self.template_vars.build_postvars(&self.posts[*index]);
                    posts_vars.push(post_vars);
                }
                template_vars.posts = Some(posts_vars);

                let mut output = models::Output {
                    visit_url: self.config.build_root_url(&current_page.current_url()),
                    output_files: vec![output_file],
                    template_vars,
                    template_file: String::from("posts.hbs"),
                };
                if i == 0 {
                    let tag_index_output_file =
                        self.config.build_dist_html_filepath(&tag.url, true);
                    output.output_files.push(tag_index_output_file);
                }

                outputs.push(output);
            }
        }
        Ok(outputs)
    }

    fn build_index(&self) -> Result<Vec<models::Output>, Box<dyn std::error::Error>> {
        // index page is same as first page of posts
        let pagination = models::Pagination::new(self.posts.len(), self.config.url.per_page_size);
        let current_page = pagination.build_each_page(1, &self.config.url.post_page_format);

        // build template vars
        let mut template_vars = self.template_vars.get_global();
        template_vars.pagination = Some(current_page.build_template_vars());
        let posts = &self.posts[current_page.start..current_page.end];
        let mut posts_vars = Vec::new();
        for p in posts {
            let post_vars = self.template_vars.build_postvars(p);
            posts_vars.push(post_vars);
        }
        template_vars.posts = Some(posts_vars);

        let output_file = self.config.build_dist_html_filepath("index.html", true);
        // set outputs
        let outputs = vec![models::Output {
            visit_url: self.config.build_root_url("index.html"),
            output_files: vec![output_file],
            template_vars,
            template_file: self.config.theme.index_template.clone(),
        }];
        Ok(outputs)
    }

    fn build_pages(&self) -> Result<Vec<models::Output>, Box<dyn std::error::Error>> {
        let mut outputs = vec![];

        // build each page
        for p in &self.pages {
            let output_file = self.config.build_dist_html_filepath(&p.slug_url, true);
            let mut template_vars = self.template_vars.get_global();
            template_vars.page = Some(self.template_vars.build_postvars(p));
            outputs.push(models::Output {
                visit_url: self.config.build_root_url(&p.slug_url),
                output_files: vec![output_file],
                template_vars,
                template_file: p.meta.template.as_ref().unwrap().clone(),
            });
        }

        Ok(outputs)
    }

    fn generate_files(
        &self,
        outputs: &Vec<models::Output>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut count = 0;
        for output in outputs {
            for file in &output.output_files {
                count += 1;
                self.theme
                    .render(&output.template_file, file, &output.template_vars)?;
            }
        }
        Ok(count)
    }

    fn copy_assets(&self) {
        let copy_dirs = self.config.build_assets_dirs();
        for (src, dst) in copy_dirs {
            if std::fs::metadata(&src).is_err() {
                debug!("Copied {}, but does not exist", src);
                continue;
            }
            std::fs::create_dir_all(&dst).unwrap();
            match copy_dir_all(&src, &dst) {
                Ok(_) => debug!("Copied {} to {}", src, dst),
                Err(e) => error!("Copy failed {} to {}: {}", src, dst, e),
            }
        }
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

pub fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
