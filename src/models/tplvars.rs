use crate::models;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorVars {
    pub name: String,
    pub bio: String,
    pub url: String,
    pub avatar: String,
    pub has_social: bool,
    pub social: Option<std::collections::HashMap<String, String>>,
}

impl AuthorVars {
    pub fn new(a: &models::Author) -> AuthorVars {
        let mut author_vars = AuthorVars {
            name: a.name.clone(),
            bio: a.bio.clone(),
            url: a.website.clone(),
            avatar: a.build_avatar_url(),
            has_social: false,
            social: a.social.clone(),
        };
        if let Some(social) = &a.social {
            if !social.is_empty() {
                author_vars.has_social = true;
            }
            for (k, v) in social {
                author_vars
                    .social
                    .as_mut()
                    .unwrap()
                    .insert(k.clone(), v.clone());
            }
        }
        author_vars
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SiteVars {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub keywords: String,
    pub language: String,
    pub author: String,
    pub root_url: String,
    pub full_url: String,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct TagVars {
    pub name: String,
    pub url: String,
    pub posts_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostVars {
    pub title: String,
    pub permalink: String,
    pub date: String,
    pub updated: String,
    pub brief: String,
    pub content: String,
    pub language: String,
    pub comments: bool,
    pub tags: Vec<TagVars>,
    pub datetime: chrono::NaiveDateTime,
    pub updated_datetime: chrono::NaiveDateTime,
    pub author: AuthorVars,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaginationVars {
    pub current: usize,
    pub prev: usize,
    pub next: usize,
    pub total: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
    pub current_url: String,
    pub prev_url: String,
    pub next_url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NavVars {
    pub name: String,
    pub url: String,
    // pub children: Option<Vec<NavVars>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GlobalVars {
    pub site: SiteVars,
    pub author: AuthorVars,
    pub navs: Vec<NavVars>,
    pub tags: Option<Vec<TagVars>>,
    pub current_tag: Option<TagVars>,
    pub pagination: Option<PaginationVars>,
    pub post: Option<PostVars>,
    pub page: Option<PostVars>,
    pub posts: Option<Vec<PostVars>>,
}

impl GlobalVars {
    pub fn new(site: &models::Site) -> GlobalVars {
        let mut vars = GlobalVars {
            site: SiteVars {
                title: site.config.site.title.clone(),
                subtitle: site.config.site.subtitle.clone(),
                description: site.config.site.description.clone(),
                keywords: site.config.site.keywords.join(","),
                language: site.config.site.language.clone(),
                author: site.config.site.author.clone(),
                root_url: site.config.build_root_url(""),
                full_url: site.config.build_full_url(""),
            },
            author: AuthorVars::default(),
            navs: site
                .config
                .nav
                .iter()
                .map(|nav| NavVars {
                    name: nav.name.clone(),
                    url: site.config.build_root_url(&nav.url),
                })
                .collect(),
            current_tag: None,
            tags: None,
            pagination: None,
            post: None,
            page: None,
            posts: None,
        };
        vars.tags = Some(
            site.tags
                .iter()
                .map(|t| TagVars {
                    name: t.name.clone(),
                    url: site.config.build_root_url(&t.url),
                    posts_count: t.posts_index.len(),
                })
                .collect(),
        );
        vars.author = AuthorVars::new(&site.config.get_default_author());
        vars
    }
}

#[derive(Debug, Default)]
pub struct TemplateVars {
    cache_tags: std::collections::HashMap<String, TagVars>,
    cache_global_vars: Option<GlobalVars>,
}

impl TemplateVars {
    pub fn new(site: &models::Site) -> TemplateVars {
        let mut vars = TemplateVars {
            cache_tags: std::collections::HashMap::new(),
            cache_global_vars: Some(GlobalVars::new(site)),
        };
        for t in &site.tags {
            let tag_vars = TagVars {
                name: t.name.clone(),
                url: site.config.build_root_url(t.url.as_str()),
                posts_count: t.posts_index.len(),
            };
            vars.cache_tags.insert(t.name.clone(), tag_vars);
        }
        vars
    }

    pub fn get_global(&self) -> GlobalVars {
        self.cache_global_vars.as_ref().unwrap().clone()
    }

    pub fn get_tag(&self, tag: &str) -> Option<TagVars> {
        self.cache_tags.get(tag).cloned()
    }

    pub fn build_postvars(&self, p: &models::Post) -> PostVars {
        let mut post_vars = PostVars {
            title: p.meta.title.clone(),
            permalink: p.slug_url.clone(),
            author: AuthorVars::new(p.author.as_ref().unwrap()),
            date: p.meta.date.clone(),
            updated: p.meta.updated.as_ref().unwrap().clone(),
            brief: p.brief_html.clone(),
            content: p.content_html.clone(),
            language: p.meta.language.as_ref().unwrap().clone(),
            comments: p.meta.comments.unwrap(),
            tags: vec![],
            datetime: p.datetime.unwrap(),
            updated_datetime: p.updated_datetime.unwrap(),
        };
        let tags = p.meta.tags.as_ref().unwrap();
        for t in tags {
            let tag = self.cache_tags.get(t).unwrap();
            post_vars.tags.push(tag.clone());
        }
        post_vars
    }
}
