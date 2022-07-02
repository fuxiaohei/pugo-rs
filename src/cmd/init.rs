use crate::models;
use log::{debug, info};

pub fn run_init() {
    info!("Initializing site...");

    // 1. create config
    let config = models::Config::default();
    config.to_file("config.toml").unwrap();
    debug!("initialized config file: config.toml");

    // 2. create site directory
    config.mkdir_all().unwrap();

    // 3. init post
    let post_file = config.build_post_uri("hello-world.md");
    create_default_post(&post_file);
    debug!("initialized default post: {}", post_file);

    // 4. init page
    let page_file = config.build_page_uri("about.md");
    create_default_page(&page_file);
    debug!("initialized default page: {}", page_file);

    // 5. init theme
    models::ThemeEmbedAssets::extract(&config.directory.themes).unwrap();

    // 6. load site to check data is correct
    // TODO: load site

    info!("Initializing success!");
}

fn create_default_post(path: &str) {
    let bytes = include_bytes!("initdata/post.md");
    let mut post = models::Post::default();
    post.meta.title = "Hello World".to_string();
    post.meta.slug = "/hello-world".to_string();
    post.meta.date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    post.meta.tags = Some(vec!["hello".to_string(), "world".to_string()]);
    post.content_markdown
        .push_str(&String::from_utf8_lossy(bytes));
    post.to_file(path).unwrap();
}

fn create_default_page(path: &str) {
    let bytes = include_bytes!("initdata/page.md");
    let mut page = models::Post::default();
    page.meta.title = "About".to_string();
    page.meta.slug = "/about".to_string();
    page.meta.date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    page.content_markdown
        .push_str(&String::from_utf8_lossy(bytes));
    page.to_file(path).unwrap();
}
