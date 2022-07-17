#[macro_use]
extern crate clap;

extern crate log;

use clap::{Parser, Subcommand};

mod cmd;
mod models;
mod utils;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Init sample site project
    Init {},
    /// Build site
    Build(cmd::BuildArgs),
    /// Serve site and watch for changes
    Serve(cmd::ServerArgs),
}

fn main() {
    // set logger
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::max())
        .format_module_path(false)
        .init();

    // parse cli arguments
    let args = Cli::parse();
    // run command
    match args.command {
        Commands::Init {} => {
            cmd::run_init();
        }
        Commands::Build(args) => {
            cmd::run_build(args);
        }
        Commands::Serve(args) => {
            cmd::run_serve(args);
        }
    }
}
