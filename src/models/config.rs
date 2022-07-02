use crate::models;

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
}
