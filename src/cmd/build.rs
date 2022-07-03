use crate::cmd;
use crate::models;
use log::{debug, error, info};

pub fn run_build(args: cmd::BuildArgs) {
    info!("Building start");

    let config_file = "config.toml";
    let site = match models::Site::load(config_file) {
        Ok(site) => site,
        Err(err) => {
            error!("Build failed: {}", err);
            return;
        }
    };

    match site.build() {
        Ok(_) => info!("Build success"),
        Err(err) => error!("Build failed: {}", err),
    }

    if args.watch {
        if args.watch_in_spawn {
            std::thread::spawn(move || {
                start_watch(&site.config.directory, &args);
            });
            debug!("Watching in spawn");
        } else {
            start_watch(&site.config.directory, &args);
        }
    }

    info!("Loaded site successfully");
}

pub fn start_watch(dir_config: &models::DirectoryConfig, build_args: &cmd::BuildArgs) {
    // TODO: finish watching
    debug!("Watching {:?}, {:?}", dir_config, build_args);
}
