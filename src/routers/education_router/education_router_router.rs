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

use actix::{Actor, Addr, StreamHandler};
use actix_web::web::{self, delete, get, post, put};
use actix_web::{HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::sync::{Arc, Mutex};

// Store WebSocket clients
type WsClients = Arc<Mutex<Vec<Addr<EducationWebSocket>>>>;

struct EducationWebSocket {
    #[allow(dead_code)] // Suppress warning for unused field
    clients: WsClients,
}

impl Actor for EducationWebSocket {
    type Context = ws::WebsocketContext<Self>; // Fixed incorrect context type
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for EducationWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            println!("Received WebSocket message: {}", text);
            ctx.text(format!("Echo: {}", text)); // Fixed message sending
        }
    }
}

// WebSocket upgrade handler
async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    clients: web::Data<WsClients>,
) -> Result<HttpResponse, actix_web::Error> {
    let ws = EducationWebSocket {
        clients: clients.get_ref().clone(),
    };

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}

// Notify WebSocket clients
fn notify_clients(clients: &WsClients, message: &str) {
    let clients_guard = clients.lock().unwrap();
    for client in clients_guard.iter() {
        client.do_send(EducationWebSocketMessage(message.to_string())); // Fixed message sending
    }
}

struct EducationWebSocketMessage(String);

impl actix::Message for EducationWebSocketMessage {
    type Result = ();
}

impl actix::Handler<EducationWebSocketMessage> for EducationWebSocket {
    type Result = ();
    fn handle(&mut self, msg: EducationWebSocketMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// Real-time GET handlers
async fn get_all_education_real_time(
    state: web::Data<AppState>,
    clients: web::Data<WsClients>,
) -> impl actix_web::Responder {
    let response = get_all_education_service(state).await;
    notify_clients(&clients, "Updated education list");
    response
}

async fn get_education_by_username_real_time(
    username: web::Path<String>,
    state: web::Data<AppState>,
    clients: web::Data<WsClients>,
) -> impl actix_web::Responder {
    let username_str = username.clone(); // Fixed move issue
    let response = get_education_by_username_service(state, username).await;
    notify_clients(
        &clients,
        &format!("Updated education for user: {}", username_str),
    );
    response
}

// Define the education routers with WebSocket support
pub fn education_routers(cfg: &mut web::ServiceConfig, state: Arc<AppState>) {
    let clients: WsClients = Arc::new(Mutex::new(Vec::<Addr<EducationWebSocket>>::new())); // Fixed type inference

    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(clients.clone())) // Pass WebSocket clients
            .route("", web::post().to(create_education_service))
            .route("", web::get().to(get_all_education_real_time)) // Real-time route
            .route(
                "/username/{username}",
                web::get().to(get_education_by_username_real_time),
            ) // Real-time route
            .route("/id/{id}", web::get().to(get_education_by_id_service))
            .route("/ws", web::get().to(ws_handler)), // WebSocket route
    );
}

// Original routers_education function (Unchanged)
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
