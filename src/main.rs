use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use config::application_conf::AppConfig;
use dotenv::dotenv;
use slog::info;

mod config;
mod controllers;
mod error;
mod handlers;
mod libs;
mod middleware;
mod models;
mod routers;

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = AppConfig::from_env().unwrap();
    let log = AppConfig::configure_log();

    info!(
        log,
        "Server is running at http:{}:{}", config.server.host, config.server.port
    );
    println!(
        "Server started at http://{}:{}",
        config.server.host, config.server.port
    );

    HttpServer::new(|| App::new().route("/", web::get().to(manual_hello)))
        .bind(format!("{}:{}", config.server.host, config.server.port))?
        .run()
        .await
}
