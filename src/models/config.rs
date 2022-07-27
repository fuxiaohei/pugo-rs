use crate::models;
use crate::utils;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
            index_template: "posts.hbs".to_string(),
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
            nav: vec![
                NavConfig {
                    name: "Archives".to_string(),
                    url: "/archives".to_string(),
                    children: None,
                },
                NavConfig {
                    name: "About".to_string(),
                    url: "/about".to_string(),
                    children: None,
                },
            ],
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

    pub fn build_full_url(&self, url: &str) -> String {
        let base_url = utils::merge_url(&self.url.base, &self.url.root);
        utils::merge_url(base_url.as_str(), url)
    }

    pub fn get_posts_dir(&self) -> String {
        utils::merge_url(&self.directory.source, "posts")
    }

    pub fn get_pages_dir(&self) -> String {
        utils::merge_url(&self.directory.source, "pages")
    }

    pub fn get_theme_dir(&self) -> String {
        utils::merge_url(self.directory.themes.as_str(), self.theme.name.as_str())
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

    pub fn get_default_author(&self) -> models::Author {
        self.get_author(&self.site.author)
    }

    pub fn get_output_dir(&self, with_root: bool) -> String {
        let mut output = String::from(&self.directory.output);
        if with_root {
            output = utils::merge_url(&self.directory.output, &self.url.root)
        }
        output
    }

    pub fn build_dist_filepath(&self, name: &str, with_root: bool) -> String {
        let mut output = self.get_output_dir(with_root);
        output = utils::merge_url(&output, name);
        output
    }

    pub fn build_dist_html_filepath(&self, name: &str, with_root: bool) -> String {
        let mut output = self.build_dist_filepath(name, with_root);
        if !output.ends_with(".html") {
            output.push_str("/index.html");
        }
        output
    }

    pub fn build_assets_dirs(&self) -> std::collections::HashMap<String, String> {
        let mut assets_dirs = std::collections::HashMap::new();
        for dir in &self.directory.assets {
            let src = utils::merge_url(&self.directory.source, dir);
            let output = utils::merge_url(&self.get_output_dir(true), dir);
            assets_dirs.insert(src, output);
        }
        for dir in &self.theme.assets_dir {
            let src = utils::merge_url(&self.get_theme_dir(), dir);
            let output = utils::merge_url(&self.get_output_dir(true), dir);
            assets_dirs.insert(src, output);
        }
        assets_dirs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_config() {
        let mut config = Config::default();
        assert_eq!(config.get_posts_dir(), "source/posts");
        assert_eq!(config.get_pages_dir(), "source/pages");
        assert_eq!(config.build_post_uri("abc"), "source/posts/abc");
        assert_eq!(config.build_page_uri("abc"), "source/pages/abc");
        assert_eq!(
            config.build_dist_html_filepath("abc", true),
            "dist/abc/index.html"
        );
        assert_eq!(config.get_theme_dir(), "themes/default");

        config.url.root = "blog".to_string();
        assert_eq!(config.build_root_url("abc"), "/blog/abc");
        assert_eq!(config.get_slug_link(), "/blog/:year/:month/:day/:slug");
        assert_eq!(
            config.build_full_url("abc"),
            "http://localhost:19292/blog/abc"
        );
        assert_eq!(config.get_output_dir(true), "dist/blog");
        assert_eq!(config.get_output_dir(false), "dist");
        assert_eq!(
            config.build_dist_filepath("abc.xml", true),
            "dist/blog/abc.xml"
        );

        let assets_dirs = config.build_assets_dirs();
        assert_eq!(assets_dirs.len(), 2);
        assert_eq!(
            assets_dirs.get("themes/default/static").unwrap(),
            "dist/blog/static"
        );
        assert_eq!(
            assets_dirs.get("source/assets").unwrap(),
            "dist/blog/assets"
        );

        assert_eq!(config.get_author("abc").name, "abc");
        assert_eq!(config.get_default_author().name, "author");
    }
}
