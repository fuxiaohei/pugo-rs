use crate::models;
use log::{debug, info};

pub struct Site {
    pub config: models::Config,
    pub posts: Vec<models::Post>,
    pub pages: Vec<models::Post>,
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

        let site = Site {
            config,
            posts,
            pages,
        };
        site.parse_source()?;
        Ok(site)
    }

    fn parse_source(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn build(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
