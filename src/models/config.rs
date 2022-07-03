use crate::models;
use crate::utils;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DirectoryConfig {
    pub source: String,
    pub output: String,
    pub themes: String,
    pub assets: Vec<String>,
}

impl DirectoryConfig {
    pub fn new() -> DirectoryConfig {
        Self {
            source: String::from("source"),
            output: String::from("dist"),
            themes: String::from("themes"),
            assets: ["assets".to_string()].to_vec(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UrlConfig {
    pub base: String,
    pub root: String,
    pub post_link_format: String,
    pub post_page_format: String,
    pub per_page_size: usize,
    pub tag_link_format: String,
    pub tag_page_format: String,
}

impl UrlConfig {
    pub fn new() -> UrlConfig {
        Self {
            base: String::from("http://localhost:19292"),
            root: String::from("/"),
            post_link_format: String::from("/:year/:month/:day/:slug"),
            post_page_format: String::from("/page/:page"),
            tag_link_format: String::from("/tag/:tag"),
            tag_page_format: String::from("/tag/:tag/page/:page"),
            per_page_size: 10,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub index_template: String,
    pub assets_dir: Vec<String>,
}

impl ThemeConfig {
    pub fn new() -> ThemeConfig {
        Self {
            name: "default".to_string(),
            index_template: "posts.html".to_string(),
            assets_dir: ["static".to_string()].to_vec(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NavConfig {
    pub name: String,
    pub url: String,
    pub children: Option<Vec<NavConfig>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SiteConfig {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub language: String,
    pub author: String,
}

impl SiteConfig {
    pub fn new() -> SiteConfig {
        Self {
            title: "PuGo".to_string(),
            subtitle: "a simple static site generator".to_string(),
            description: "Build your content into static websites and blogs".to_string(),
            keywords: [
                "rust".to_string(),
                "blog".to_string(),
                "static".to_string(),
                "website".to_string(),
                "generator".to_string(),
            ]
            .to_vec(),
            language: "en".to_string(),
            author: "pugo".to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub site: SiteConfig,
    pub url: UrlConfig,
    pub directory: DirectoryConfig,
    pub theme: ThemeConfig,
    pub nav: Vec<NavConfig>,
    pub author: Option<std::collections::HashMap<String, models::Author>>,
}

impl Config {
    pub fn default() -> Self {
        let mut cfg = Config {
            site: SiteConfig::new(),
            url: UrlConfig::new(),
            directory: DirectoryConfig::new(),
            theme: ThemeConfig::new(),
            nav: vec![NavConfig {
                name: "About".to_string(),
                url: "/about".to_string(),
                children: None,
            }],
            author: Some(std::collections::HashMap::new()),
        };
        let author = models::Author::default();
        cfg.author
            .as_mut()
            .unwrap()
            .insert("pugo".to_string(), author);
        cfg
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = toml::to_string_pretty(&self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        let cfg = toml::from_slice(&bytes)?;
        Ok(cfg)
    }

    pub fn build_post_uri(&self, filename: &str) -> String {
        let mut uri = utils::merge_url(&self.directory.source, "posts");
        uri = utils::merge_url(&uri, filename);
        uri
    }

    pub fn build_root_url(&self, url: &str) -> String {
        let root_url = utils::merge_url(&self.url.root, url);
        if !root_url.starts_with('/') {
            format!("/{}", root_url)
        } else {
            root_url
        }
    }

    pub fn get_posts_dir(&self) -> String {
        utils::merge_url(&self.directory.source, "posts")
    }

    pub fn get_pages_dir(&self) -> String {
        utils::merge_url(&self.directory.source, "pages")
    }

    pub fn get_slug_link(&self) -> String {
        self.build_root_url(&self.url.post_link_format)
    }

    pub fn build_page_uri(&self, filename: &str) -> String {
        let mut uri = utils::merge_url(&self.directory.source, "pages");
        uri = utils::merge_url(&uri, filename);
        uri
    }

    pub fn mkdir_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&self.directory.source)?;
        std::fs::create_dir_all(&self.directory.themes)?;
        std::fs::create_dir_all(&self.directory.output)?;
        std::fs::create_dir_all(utils::merge_url(&self.directory.source, "posts"))?;
        std::fs::create_dir_all(utils::merge_url(&self.directory.source, "pages"))?;
        for asset in &self.directory.assets {
            std::fs::create_dir_all(utils::merge_url(&self.directory.source, asset))?;
        }
        Ok(())
    }

    pub fn get_author(&self, name: &str) -> models::Author {
        if self.author.is_none() {
            return models::Author::create_by_name(name);
        }
        let author = self.author.as_ref().unwrap().get(name);
        if author.is_none() {
            return models::Author::create_by_name(name);
        }
        author.unwrap().clone()
    }
}
