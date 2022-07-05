use crate::cmd;
use crate::models;
use log::{debug, error, info};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Sender};

pub fn run_build(args: cmd::BuildArgs) {
    let start = std::time::Instant::now();
    run_build_site(args).unwrap();
    info!("Loaded site successfully, eplased: {:?}", start.elapsed());
}

pub fn run_build_site(
    args: cmd::BuildArgs,
) -> Result<models::Site<'static>, Box<dyn std::error::Error>> {
    info!("Building start");

    let config_file = "config.toml";
    let site = match models::Site::load(config_file) {
        Ok(site) => site,
        Err(err) => {
            error!("Build failed: {}", err);
            return Err(err);
        }
    };

    match site.build() {
        Ok(_) => info!("Build success"),
        Err(err) => error!("Build failed: {}", err),
    }

    let directory_config = site.config.directory.clone();
    if args.watch {
        if args.watch_in_spawn {
            std::thread::spawn(move || {
                start_watch(&directory_config, &args);
            });
            debug!("Watching in spawn");
        } else {
            start_watch(&directory_config, &args);
        }
    }
    Ok(site)
}

pub fn start_watch(dir_config: &models::DirectoryConfig, build_args: &cmd::BuildArgs) {
    let (send, recv) = channel();
    let dirs = vec![dir_config.source.to_string(), dir_config.themes.to_string()];
    let args = build_args.clone();

    // use time ticker to handle duplicated events
    std::thread::spawn(move || {
        let ticker = crossbeam_channel::tick(std::time::Duration::from_millis(1000));
        loop {
            let mut is_recv_ok = false;
            ticker.recv().unwrap();
            while recv.try_recv().is_ok() {
                is_recv_ok = true;
            }
            if is_recv_ok {
                info!("Watching triggered");
                cmd::run_build_site(args).unwrap();
            }
        }
    });

    // start watching
    if start_watching_dirs(&dirs, &send).is_err() {
        error!("Start watching site failed");
    }
}

pub fn start_watching_dirs(dirs: &Vec<String>, sender: &Sender<String>) -> notify::Result<()> {
    info!("Watching site");
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx)?;
    for dir in dirs {
        debug!("Watching source directory: {}", dir);
        watcher.watch(std::path::Path::new(&dir), RecursiveMode::Recursive)?;
    }

    for res in rx {
        match res {
            Ok(event) => {
                event.paths.iter().for_each(|path| {
                    sender.send(path.to_str().unwrap().to_string()).unwrap();
                });
            }
            Err(e) => debug!("Watching error: {:?}", e),
        }
    }

    Ok(())
}
