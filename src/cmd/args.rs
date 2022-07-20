use clap::Args;

#[derive(Debug, Args, Clone, Copy)]
pub struct BuildArgs {
    /// Watching for changes
    #[clap(short = 'w', long)]
    pub watch: bool,
    /// not clap argument
    #[clap(skip)]
    pub watch_in_spawn: bool,
    /// Clean old files before current build
    #[clap(short = 'c', long)]
    pub clean: bool,
    /// Compress built files to one tar.gz
    #[clap(short = 'a', long)]
    pub archive: bool,
}

#[derive(Debug, Args)]
pub struct ServerArgs {
    /// Set http port
    #[clap(short = 'p', long, default_value = "19292")]
    pub port: u16,
    /// Clean old files before current build
    #[clap(short = 'c', long)]
    pub clean: bool,
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    #[clap(value_parser)]
    pub path: String,
    #[clap(short = 'p', long)]
    pub page: bool,
}
