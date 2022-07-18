use crate::cmd;
use actix_files::Files;
use actix_web::{App, HttpServer};
use log::info;

pub fn run_serve(args: cmd::ServerArgs) {
    let build_args = cmd::BuildArgs {
        watch: true,
        watch_in_spawn: true,
        archive: false,
        clean: args.clean,
    };
    // build first, then start server
    let site = cmd::run_build_site(build_args).unwrap();
    start_server(site.config.directory.output, args.port).unwrap();
}

#[actix_web::main]
async fn start_server(dst_dir: String, port: u16) -> std::io::Result<()> {
    info!("Starting server at http://localhost:{}", port);
    HttpServer::new(move || {App::new().service(Files::new("/", &dst_dir).index_file("index.html"))})
        .bind(("127.0.0.1", port))?
        .run()
        .await
}
