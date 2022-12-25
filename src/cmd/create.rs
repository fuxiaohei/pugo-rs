use crate::cmd;
use crate::models;

use log::{debug, info};
pub fn run_create(args: cmd::CreateArgs) {
    info!("Create new post or page");

    // 1. load config to get source directory
    let path = "config.toml";
    let config = models::Config::from_file(path).unwrap();
    debug!("Loaded config: {}", path);

    // 2. create output file
    let mut content_dir = config.get_posts_dir();
    if args.page {
        content_dir = config.get_pages_dir();
    }
    let slug = args.path.trim_end_matches(".md").replace('/', "-");
    let mut path = std::path::PathBuf::new();
    path.push(content_dir);
    path.push(args.path);
    // get basename as post title
    let basename = path.file_stem().unwrap().to_str().unwrap();
    if args.page {
        create_empty_page(path.to_str().unwrap(), basename, &slug);
        info!("Created page: {}", path.to_str().unwrap());
        return;
    }
    create_empty_post(path.to_str().unwrap(), basename, &slug);
    info!("Created post: {}", path.to_str().unwrap());
}

fn create_empty_post(path: &str, title: &str, slug: &str) {
    let bytes = include_bytes!("initdata/new_post.md");
    let mut post = models::Post::default();
    post.meta.title = title.to_string();
    post.meta.slug = slug.to_string();
    post.meta.date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    post.meta.tags = Some(vec![]);
    post.content_markdown
        .push_str(&String::from_utf8_lossy(bytes));
    post.to_file(path).unwrap();
}

fn create_empty_page(path: &str, title: &str, slug: &str) {
    let bytes = include_bytes!("initdata/new_post.md");
    let mut page = models::Post::default();
    page.meta.title = title.to_string();
    page.meta.slug = slug.to_string();
    page.meta.date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    page.meta.template = Some("page.hbs".to_string());
    page.content_markdown
        .push_str(&String::from_utf8_lossy(bytes));
    page.to_file(path).unwrap();
}
