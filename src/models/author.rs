#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
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
    pub fn create_by_name(name: &str) -> Author {
        Author {
            name: name.to_string(),
            ..Author::default()
        }
    }
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
    pub fn build_avatar_url(&self) -> String {
        use md5::{Digest, Md5};
        if self.use_gravatar {
            let mut url = "https://www.gravatar.com/avatar/".to_string();
            let mut hasher = Md5::new();
            hasher.update(&self.email);
            let result = hasher.finalize();
            let hex_hash = base16ct::lower::encode_string(&result);
            url.push_str(hex_hash.as_str());
            url
        } else {
            self.avatar.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_avatar_url() {
        let mut author = Author::default();
        assert_eq!(author.name, "writer");
        assert_eq!(
            author.build_avatar_url(),
            "https://www.gravatar.com/avatar/876f6a6c274d1f94238c26ce721dcc31"
        );

        author.use_gravatar = false;
        author.avatar = String::from("avatar.jpg");
        assert_eq!(author.build_avatar_url(), "avatar.jpg");
    }
}
