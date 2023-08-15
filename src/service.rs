use crate::api::{File, FileContent, FileList, NewFile, RootHash};
use crate::storage::Storage;
use actix_web::{get, post, web, HttpResponse, Responder};
use std::ops::Deref;
use std::sync::Mutex;

#[get("/files")]
pub async fn get_file_list(storage: web::Data<Mutex<Storage>>) -> impl Responder {
    let files = storage
        .lock()
        .expect("should lock")
        .list_all_files()
        .into_iter()
        .map(|(i, name, _)| File { id: i as u32, name })
        .collect();
    HttpResponse::Ok().json(FileList { files })
}

#[post("/files")]
pub async fn upload_new_file(
    storage: web::Data<Mutex<Storage>>,
    new_file: web::Json<NewFile>,
) -> impl Responder {
    let NewFile { name, content } = new_file.0;
    let id = storage
        .lock()
        .expect("should lock")
        .add_new_file(name.clone(), content);
    HttpResponse::Created().json(File {
        name,
        id: id as u32,
    })
}

#[get("/files/{id}")]
pub async fn get_file_content(
    storage: web::Data<Mutex<Storage>>,
    id: web::Path<u32>,
) -> impl Responder {
    let id = *id.deref();
    let content = storage
        .lock()
        .expect("should lock")
        .get_file_by_id(id as usize);
    match content {
        Some((name, content, proof)) => HttpResponse::Ok().json(FileContent {
            id,
            name,
            content,
            proof,
        }),
        None => HttpResponse::NotFound().body(format!("file {} not found", id)),
    }
}

#[get("/root")]
pub async fn get_tree_root(storage: web::Data<Mutex<Storage>>) -> impl Responder {
    let maybe_root = storage.lock().expect("should lock").root_hash();
    match maybe_root {
        Some(hash) => HttpResponse::Ok().json(RootHash { hash }),
        None => {
            HttpResponse::NotFound().body("root is not available yet - try uploading some files")
        }
    }
}
