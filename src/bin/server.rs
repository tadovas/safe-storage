use actix_web::{web, App, HttpServer};
use clap::Parser;
use safe_storage::service::{get_file_content, get_file_list, get_tree_root, upload_new_file};
use safe_storage::storage::Storage;
use std::sync::Mutex;

/// A merkle tree based "secure" storage service to upload files and download any of them later
/// with merkle proof for verification
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CmdArgs {
    /// listen for incoming requests on given port
    #[arg(short, long, value_name = "port", default_value_t = 8080)]
    listen_port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cmd_args = CmdArgs::parse();

    let storage = web::Data::new(Mutex::new(Storage::new()));
    HttpServer::new(move || {
        App::new()
            .app_data(storage.clone())
            .service(get_file_list)
            .service(upload_new_file)
            .service(get_file_content)
            .service(get_tree_root)
    })
    .bind(("0.0.0.0", cmd_args.listen_port))?
    .run()
    .await
}
