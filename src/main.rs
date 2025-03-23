use crate::routers::all_routers::all_routers;
use actix::{Actor, StreamHandler};
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_actors::ws;
use config::application_conf::AppConfig;
use dotenv::dotenv;
use libs::db::conn_db::ConnDb;
use std::sync::Arc;

mod config;
mod controllers;
mod error;
mod handlers;
mod libs;
mod models;
mod routers;
mod services;

#[derive(Debug)]
pub struct AppState {
    pub db: ConnDb,
}

struct WebSocketConnection;

impl Actor for WebSocketConnection {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("Received: {}", text);
                ctx.text(format!("Echo: {}", text));
            }
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            _ => (),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = AppConfig::from_env().unwrap();
    let app_state = Arc::new(AppState {
        db: ConnDb::init().await.unwrap(),
    });

    println!(
        "Server started at http://{}:{}",
        config.server.host, config.server.port
    );

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::from(app_state.clone()))
            .configure(|cfg| all_routers(cfg, app_state.clone()))
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
