pub fn merge_url(base: &str, url: &str) -> String {
    let mut merged = String::from(base.trim_end_matches('/'));
    if !url.starts_with('/') {
        merged.push('/');
    }
    merged.push_str(url);
    merged
}
