use crate::models;
use log::{debug, info};

pub fn run_init() {
    info!("Initilizing site...");

    // 1. create config
    let config = models::Config::default();
    config.to_file("config.toml").unwrap();
}
