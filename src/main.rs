// Many service/repository/guard helpers are defined ahead of being wired into
// routes. Silence dead-code noise crate-wide instead of deleting future-use API.
#![allow(dead_code)]

mod api;
mod config;
mod controller;
mod domain;
mod errors;
mod guards;
mod handler;
mod helpers;
mod mappers;
mod middleware;
mod models;
mod pipeline;
mod repositories;
mod schema;
mod services;
mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    config::logger::init();

    let port = env::var("PORT").unwrap_or_else(|_| "4646".to_string());
    let address = format!("0.0.0.0:{port}");

    let pg_manager = config::db::init_postgres_manager()
        .await
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
    pg_manager
        .run_migrations()
        .await
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
    let state = web::Data::new(config::state::AppState::new(pg_manager.clone()));

    println!("🚀 Space-Together backend starting on {address}");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(crate::middleware::logging::RequestLoggingMiddleware)
            .wrap(cors)
            .wrap(crate::middleware::tenant_middleware::TenantMiddleware::new())
            .app_data(state.clone())
            .configure(api::init_routes)
    })
    .bind(address)?
    .run()
    .await
}
