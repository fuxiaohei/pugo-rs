#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub website: String,
    pub bio: String,
    pub avatar: String,
    pub use_gravatar: bool,
    pub social: Option<std::collections::HashMap<String, String>>,
}

impl Author {
    pub fn default() -> Self {
        let mut author = Author {
            name: "writer".to_string(),
            email: "writer@example.com".to_string(),
            website: "https://pugo.io".to_string(),
            bio: "sample author of PuGo".to_string(),
            avatar: "".to_string(),
            use_gravatar: true,
            social: Some(std::collections::HashMap::new()),
        };
        author.social.as_mut().unwrap().insert(
            "github".to_string(),
            "https://github.com/fuxiaohei/pugo-rs".to_string(),
        );
        author
    }
}
