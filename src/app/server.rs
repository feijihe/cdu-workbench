use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use log::{info, warn};
use std::path::PathBuf;
use crate::utils::datetime;

#[allow(dead_code)]
const STATIC_FILES_PATH: &str = "./dist/";
#[allow(dead_code)]
const SERVER_ADDRESS: &str = "0.0.0.0:8080";

pub struct Server;

impl Server {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn run(&self, host: &str, port: &str) -> std::io::Result<()> {
        // let rt = tokio::runtime::Builder::new_current_thread()
        //     .enable_all()
        //     .build()?;
        
        // rt.block_on(main())
        let server_address = format!("{}:{}", host, port);
        env_logger::init();
        
        // 使用 get_current_time 获取格式化的时间戳
        let current_time = datetime::get_current_time();
        info!("Starting server on http://{}", server_address);
        println!("[{}] [INFO] Running on http://{} (CTRL + C to quit)", current_time, server_address);

        let static_path = PathBuf::from(STATIC_FILES_PATH);
        if !static_path.exists() {
            warn!("Static files directory not found at: {:?}", static_path);
        }

        HttpServer::new(|| {
            App::new()
                .service(Files::new("/", STATIC_FILES_PATH).index_file("index.html"))
                .service(web::resource("/cdu/{tail:.*}").route(web::get().to(handle_api)))
                .default_service(web::to(spa_index))
        })
        .bind(SERVER_ADDRESS)?
        .run()
        .await
    }
}

#[allow(dead_code)]
async fn spa_index() -> HttpResponse {
    let index_path = PathBuf::from(STATIC_FILES_PATH).join("index.html");

    match std::fs::read_to_string(index_path) {
        Ok(content) => HttpResponse::Ok().content_type("text/html").body(content),
        Err(_) => HttpResponse::InternalServerError().body("Failed to read index.html"), // Handle the error as needed
    }
}

// fn api_index() -> HttpResponse {
//     HttpResponse::Ok().body("api index")
// }

async fn handle_api(_req: actix_web::HttpRequest, path: web::Path<String>) -> impl actix_web::Responder {
    let api_path = path.into_inner();
    // let client = reqwest::Client::new();
    
    // 记录请求时间
    let request_time = datetime::get_current_time();
    info!("API request at {}: {}", request_time, api_path);

    HttpResponse::Ok().body("api index")
}