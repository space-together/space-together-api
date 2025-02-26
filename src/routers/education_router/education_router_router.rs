use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::education_handle::education_handle_handle::{
        create_education_handle, delete_education_by_id_handle, get_all_education_handle,
        get_education_by_id_handle, get_education_by_username_handle,
        update_education_by_id_handle,
    },
    services::education_services::{
        create_education_service, get_all_education_service, get_education_by_id_service,
        get_education_by_username_service,
    },
    AppState,
};
use actix::{Actor, StreamHandler};
use actix_web::{HttpRequest, HttpResponse};
use actix_web_actors::ws;
// version 2

struct EducationWebSocket;

impl Actor for EducationWebSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for EducationWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("Received WebSocket message: {}", text);
                ctx.text(format!("Echo: {}", text));
            }
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            _ => (),
        }
    }
}

// WebSocket upgrade handler
async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    ws::start(EducationWebSocket {}, &req, stream)
}

// Define the education routers with WebSocket support
pub fn education_routers(cfg: &mut web::ServiceConfig, state: Arc<AppState>) {
    cfg.service(
        web::scope("/education")
            .app_data(web::Data::new(state.clone()))
            .route("", web::post().to(create_education_service))
            .route("", web::get().to(get_all_education_service))
            .route(
                "/username/{username}",
                web::get().to(get_education_by_username_service),
            )
            .route("/id/{id}", web::get().to(get_education_by_id_service))
            .route("/ws", web::get().to(ws_handler)), // WebSocket route
    );
}

pub fn routers_education(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_education_handle))
            .route("", get().to(get_all_education_handle))
            .route(
                "/username/{username}",
                get().to(get_education_by_username_handle),
            )
            .route("/{id}", get().to(get_education_by_id_handle))
            .route("/{id}", delete().to(delete_education_by_id_handle))
            .route("{id}", put().to(update_education_by_id_handle)),
    )
}
