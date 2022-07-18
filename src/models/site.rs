use crate::models;
use chrono::{Local, TimeZone, Utc};
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

        // 4. build index and archives
        outputs.extend(self.build_index()?);
        outputs.extend(self.build_archives()?);
        outputs.extend(self.build_404_page()?);

        // 5. build rss
        outputs.extend(self.build_rss()?);

        // 5. generate files
        let generated_count = self.generate_files(&mut outputs)?;

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

            let dt = Utc.from_local_datetime(&p.datetime.unwrap()).unwrap();
            outputs.push(models::Output {
                visit_url: self.config.build_root_url(&p.slug_url),
                output_files: vec![output_file],
                template_vars,
                template_file: p.meta.template.as_ref().unwrap().clone(),
                file_content: "".to_string(),
                lastmod: dt,
                sitemap_priority: 0.8,
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
            let dt = Utc
                .from_local_datetime(&posts[0].datetime.unwrap())
                .unwrap();
            outputs.push(models::Output {
                visit_url: self.config.build_root_url(&current_page.current_url()),
                output_files: vec![output_file],
                template_vars,
                template_file: String::from("posts.hbs"),
                file_content: "".to_string(),
                lastmod: dt,
                sitemap_priority: 0.7,
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

                let dt = Utc
                    .from_local_datetime(&self.posts[tag.posts_index[0]].datetime.unwrap())
                    .unwrap();
                let mut output = models::Output {
                    visit_url: self.config.build_root_url(&current_page.current_url()),
                    output_files: vec![output_file],
                    template_vars,
                    template_file: String::from("posts.hbs"),
                    file_content: "".to_string(),
                    lastmod: dt,
                    sitemap_priority: 0.7,
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
        let dt = Utc
            .from_local_datetime(&posts[0].datetime.unwrap())
            .unwrap();
        let outputs = vec![models::Output {
            visit_url: self.config.build_root_url("index.html"),
            output_files: vec![output_file],
            template_vars,
            template_file: self.config.theme.index_template.clone(),
            file_content: "".to_string(),
            lastmod: dt,
            sitemap_priority: 1.0,
        }];
        Ok(outputs)
    }

    fn build_404_page(&self) -> Result<Vec<models::Output>, Box<dyn std::error::Error>> {
        let template_vars = self.template_vars.get_global();
        let output_file = self.config.build_dist_html_filepath("404", true);
        let dt = Utc
            .from_local_datetime(&self.posts[0].datetime.unwrap())
            .unwrap();
        let outputs = vec![models::Output {
            visit_url: self.config.build_root_url("404"),
            output_files: vec![output_file],
            template_vars,
            template_file: "404.hbs".to_string(),
            file_content: "".to_string(),
            lastmod: dt,
            sitemap_priority: 0.0,
        }];
        Ok(outputs)
    }

    fn build_archives(&self) -> Result<Vec<models::Output>, Box<dyn std::error::Error>> {
        let archives = models::Archive::parse(&self.posts);
        let mut archive_vars = vec![];
        for archive in &archives {
            let mut posts_vars = vec![];
            for idx in &archive.posts_index {
                let post_vars = self.template_vars.build_postvars(&self.posts[*idx]);
                posts_vars.push(post_vars);
            }
            let archive_var = models::ArchiveVars {
                year: archive.year.clone(),
                posts: posts_vars,
            };
            archive_vars.push(archive_var);
        }

        let mut template_vars = self.template_vars.get_global();
        template_vars.archives = Some(archive_vars);

        let dt = Utc
            .from_local_datetime(&self.posts[0].datetime.unwrap())
            .unwrap();
        let output_file = self.config.build_dist_html_filepath("archives", true);
        let outputs = vec![models::Output {
            visit_url: self.config.build_root_url("archives"),
            output_files: vec![output_file],
            template_vars,
            template_file: "archives.hbs".to_string(),
            file_content: "".to_string(),
            lastmod: dt,
            sitemap_priority: 0.6,
        }];
        Ok(outputs)
    }

    fn build_rss(&self) -> Result<Vec<models::Output>, Box<dyn std::error::Error>> {
        use rss::{ChannelBuilder, ItemBuilder};
        // add post items
        let mut items = Vec::new();
        for post in &self.posts {
            let dt = Local.from_local_datetime(&post.datetime.unwrap()).unwrap();
            let full_link = self.config.build_full_url(&post.slug_url);
            let item = ItemBuilder::default()
                .title(post.meta.title.clone())
                .link(full_link)
                .content(post.content_html.clone())
                .pub_date(dt.to_rfc2822())
                .build();
            items.push(item);
        }
        // build channel
        let channel = ChannelBuilder::default()
            .title(self.config.site.title.as_str())
            .link(self.config.build_full_url(""))
            .items(items)
            .description(self.config.site.description.as_str())
            .build();

        // set outputs
        let output_url = "atom.xml";
        let output_file = self.config.build_dist_filepath(output_url, true);
        let dt = Utc
            .from_local_datetime(&self.posts[0].datetime.unwrap())
            .unwrap();
        let outputs = vec![models::Output {
            visit_url: self.config.build_root_url(output_url),
            output_files: vec![output_file],
            template_vars: self.template_vars.get_global(),
            template_file: "".to_string(),
            file_content: channel.to_string(),
            lastmod: dt,
            sitemap_priority: 0.8,
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
            let dt = Utc.from_local_datetime(&p.datetime.unwrap()).unwrap();
            outputs.push(models::Output {
                visit_url: self.config.build_root_url(&p.slug_url),
                output_files: vec![output_file],
                template_vars,
                template_file: p.meta.template.as_ref().unwrap().clone(),
                file_content: "".to_string(),
                lastmod: dt,
                sitemap_priority: 0.7,
            });
        }

        Ok(outputs)
    }

    fn generate_files(
        &self,
        outputs: &mut Vec<models::Output>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut count = 0;

        // generate sitemap
        let mut urls = Vec::new();
        for output in outputs.iter_mut() {
            // if sitemap priority is set to 0.0, skip it
            if output.sitemap_priority < 0.01 {
                continue;
            }
            let entry = sitewriter::UrlEntry {
                loc: self
                    .config
                    .build_full_url(&output.visit_url)
                    .parse()
                    .unwrap(),
                changefreq: Some(sitewriter::ChangeFreq::Weekly),
                priority: Some(output.sitemap_priority),
                lastmod: Some(output.lastmod),
            };
            urls.push(entry);
        }
        let sitemap_outputfile = self.config.build_dist_filepath("sitemap.xml", true);
        let dt = Utc
            .from_local_datetime(&self.posts[0].datetime.unwrap())
            .unwrap();
        let sitemap_output = models::Output {
            visit_url: self.config.build_root_url("sitemap.xml"),
            output_files: vec![sitemap_outputfile],
            template_vars: self.template_vars.get_global(),
            template_file: "".to_string(),
            file_content: sitewriter::generate_str(&urls),
            lastmod: dt,
            sitemap_priority: 0.0,
        };
        outputs.push(sitemap_output);

        // generate output files
        for output in outputs {
            for file in &output.output_files {
                count += 1;
                debug!("Generated file: {}", file);
                // write file content directly
                if !output.file_content.is_empty() {
                    let output_dir = std::path::Path::new(file).parent().unwrap();
                    std::fs::create_dir_all(output_dir)?;
                    std::fs::write(file, &output.file_content)?;
                } else {
                    self.theme
                        .render(&output.template_file, file, &output.template_vars)?;
                }
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

    pub fn archive(&self) -> Result<String, Box<dyn std::error::Error>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let outputdir = self.config.get_output_dir(false);
        let filename = chrono::Local::now()
            .format("%Y-%m-%d-%H-%M-%S.tar.gz")
            .to_string();
        let tar_gz = std::fs::File::create(&filename)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);
        tar.append_dir_all(&outputdir, &outputdir)?;
        tar.finish()?;

        debug!("Create archive: {}", filename);
        Ok(filename)
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
