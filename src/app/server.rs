use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use log::{info, warn};
use std::path::PathBuf;

const STATIC_FILES_PATH: &str = "./dist/";
const SERVER_ADDRESS: &str = "0.0.0.0:8080";

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init();

    info!("Starting server on http://{}", SERVER_ADDRESS);

    // Check if static files directory exists
    let static_path = PathBuf::from(STATIC_FILES_PATH);
    if !static_path.exists() {
        warn!("Static files directory not found at: {:?}", static_path);
    }

    HttpServer::new(|| {
        App::new()
            .service(Files::new("/", STATIC_FILES_PATH).index_file("index.html"))
            .default_service(web::to(spa_index))
    })
    .bind(SERVER_ADDRESS)?
    .run()
    .await
}

async fn spa_index() -> HttpResponse {
    let index_path = PathBuf::from(STATIC_FILES_PATH).join("index.html");

    match std::fs::read_to_string(index_path) {
        Ok(content) => HttpResponse::Ok().content_type("text/html").body(content),
        Err(_) => HttpResponse::InternalServerError().body("Failed to read index.html"), // Handle the error as needed
    }
}
