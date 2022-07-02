use log::debug;
use std::path::Path;

#[derive(rust_embed::RustEmbed)]
#[folder = "themes/"]
pub struct ThemeEmbedAssets;

impl ThemeEmbedAssets {
    pub fn extract(themes_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        for file in ThemeEmbedAssets::iter() {
            let tpl_file = Path::new(themes_dir).join(file.as_ref());
            let tpl_dir = tpl_file.parent().unwrap();
            std::fs::create_dir_all(tpl_dir)?;
            let asset_file = ThemeEmbedAssets::get(file.as_ref()).unwrap();
            std::fs::write(&tpl_file, asset_file.data).unwrap();
            debug!("extracted theme file: {}", tpl_file.to_str().unwrap());
        }
        Ok(())
    }
}
