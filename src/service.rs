use crate::api::{File, FileContent, FileList, NewFile, Proof};
use crate::storage::Storage;
use actix_web::{get, post, web, HttpResponse, Responder};
use std::ops::Deref;

#[get("/files")]
pub async fn get_file_list(storage: web::Data<Storage>) -> impl Responder {
    let files = storage
        .list_all_files()
        .into_iter()
        .map(|(i, name, _)| File { id: i as u32, name })
        .collect();
    HttpResponse::Ok().json(FileList { files })
}

#[post("/files")]
pub async fn upload_new_file(
    storage: web::Data<Storage>,
    new_file: web::Json<NewFile>,
) -> impl Responder {
    let NewFile { name, content } = new_file.0;
    let id = storage.add_new_file(name.clone(), content);
    HttpResponse::Created().json(File {
        name,
        id: id as u32,
    })
}

#[get("/files/{id}")]
pub async fn get_file_content(storage: web::Data<Storage>, id: web::Path<u32>) -> impl Responder {
    let id = *id.deref();
    let content = storage.get_file_by_id(id as usize);
    match content {
        Some((name, content)) => HttpResponse::Ok().json(FileContent {
            id,
            name,
            content,
            proof: Proof {},
        }),
        None => HttpResponse::NotFound().body(format!("file {} not found", id)),
    }
}
