use actix_web::{web, App, HttpServer};
use safe_storage::service::{get_file_content, get_file_list, upload_new_file};
use safe_storage::storage::Storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(Storage::new()))
            .service(get_file_list)
            .service(upload_new_file)
            .service(get_file_content)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
