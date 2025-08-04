// src/main.rs
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use sled::Db;
use std::sync::Arc;

mod handlers;
mod models;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_path = utils::get_sqlite_path().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let db = Arc::new(sled::open(db_path).unwrap());

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db.clone()))
            .route("/", web::get().to(handlers::index))
            .route("/rule", web::post().to(handlers::create_rule))
            .route("/credentials", web::get().to(handlers::credentials_page))
            .route("/scheduler", web::get().to(handlers::scheduler))
            .route("/set_credentials", web::post().to(handlers::set_credentials))
    })
    .bind(("127.0.0.1", 8082))?
    .run()
    .await
}