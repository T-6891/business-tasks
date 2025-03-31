mod api;
mod db;
mod models;

use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, get};
use dotenv::dotenv;
use std::sync::Arc;

// Временные заглушки, которые нужно заменить на реальный код
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Система управления бизнес-поручениями")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Запуск сервера на http://127.0.0.1:8080");
    
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(fs::Files::new("/static", "static").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
