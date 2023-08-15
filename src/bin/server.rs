use actix_web::{web, App, HttpServer};
use safe_storage::service::{get_file_content, get_file_list, get_tree_root, upload_new_file};
use safe_storage::storage::Storage;
use std::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let storage = web::Data::new(Mutex::new(Storage::new()));
    HttpServer::new(move || {
        App::new()
            .app_data(storage.clone())
            .service(get_file_list)
            .service(upload_new_file)
            .service(get_file_content)
            .service(get_tree_root)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
