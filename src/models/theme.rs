use crate::models;
use chrono::NaiveDateTime;
use handlebars::handlebars_helper;
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

// define date helper
handlebars_helper!(date_format: |dt: NaiveDateTime,{fmt:str = "%Y-%m-%d"} | dt.format(fmt).to_string());

pub struct Theme<'a> {
    pub dir: String,
    reg: handlebars::Handlebars<'a>,
}

impl Theme<'_> {
    pub fn parse(dir: &str) -> Result<Theme<'static>, Box<dyn std::error::Error>> {
        let mut reg = handlebars::Handlebars::new();
        reg.register_helper("date_format", Box::new(date_format));
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry.unwrap();
            let template_file_path = entry.path();
            let template_file_path_str = template_file_path.to_str().unwrap();
            if template_file_path.is_file()
                && (template_file_path.extension().unwrap() == "html"
                    || template_file_path.extension().unwrap() == "hbs")
            {
                let template_name = template_file_path
                    .strip_prefix(dir)
                    .unwrap()
                    .to_str()
                    .unwrap();
                reg.register_template_file(template_name, template_file_path_str)?;
                debug!("Loaded template: {}", template_file_path_str);
            }
        }
        Ok(Theme {
            dir: String::from(dir),
            reg,
        })
    }
    pub fn render(
        &self,
        name: &str,
        to_file: &str,
        vars: &models::GlobalVars,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_dir = std::path::Path::new(to_file).parent().unwrap();
        std::fs::create_dir_all(output_dir)?;
        let mut output_file = std::fs::File::create(to_file)?;
        self.reg.render_to_write(name, &vars, &mut output_file)?;
        Ok(())
    }
}
