use actix_web::{
    web::{self, scope, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};
use actix_web_actors::ws;
use std::sync::Arc;

use super::{
    auth_router::{
        delete_user_session_router, get_user_session_router, oauth2_provider_url_router,
        update_user_session_expires_router, user_login_router, user_register_router,
    },
    class_router::{
        activities_type_router::routers_activities_type,
        activity_router::routers_activity,
        class_group_router::routers_class_group,
        class_room_router::{main_class_router, routers_class_room},
        class_router_router::routers_class,
        class_type_router::routers_class_type,
    },
    conversation_router::{
        conversation_router_router::routers_conversation, message_router::routers_message,
    },
    database_router::database_status_router::routers_database,
    education_router::education_router_router::{education_routers, routers_education},
    file_router::{file_router_router::routers_file, file_type_route::routers_file_type},
    request_router::{
        request_router_router::routers_request, request_type_router::routers_request_type,
    },
    school_router::{
        school_router_auth::routers_school_auth, school_router_router::routers_school,
        sector_router::routers_sector, trade_router::routers_trade,
    },
    subject_router::{
        subject_router_router::{routers_subject, subject_routers},
        subject_type_router::routers_subject_type,
    },
    user_router::{user_role_router::routers_user_role, user_router_router::routers_user},
};
use crate::{
    handlers::database_handle::all_end_point_handle::list_all_endpoints, AppState,
    WebSocketConnection,
};

pub fn all_routers(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::resource("/").route(web::get().to(welcome)));
    cfg.service(
        scope("/api/v0.0.1")
            .route("/", web::get().to(manual_hello))
            .route("/endpoints", web::get().to(list_all_endpoints)) // Debug route
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/users").configure(|user_cfg| {
                routers_user_role(user_cfg, state.clone());
                routers_user(user_cfg, state.clone());
            }))
            .service(web::scope("/classes/room").configure(|user_cfg| {
                routers_class_room(user_cfg, state.clone());
            }))
            .service(scope("/classes/activities").configure(|user_cfg| {
                routers_activities_type(user_cfg, state.clone());
                routers_activity(user_cfg, state.clone());
            }))
            .service(web::scope("/classes").configure(|user_cfg| {
                routers_class_type(user_cfg, state.clone());
                routers_class_group(user_cfg, state.clone());
                routers_class(user_cfg, state.clone());
            }))
            .service(web::scope("/subject").configure(|user_cfg| {
                routers_subject_type(user_cfg, state.clone());
                routers_subject(user_cfg, state.clone());
            }))
            .service(web::scope("/conversations").configure(|user_cfg| {
                routers_message(user_cfg, state.clone());
                routers_conversation(user_cfg, state.clone());
            }))
            .service(web::scope("/db").configure(|user_cfg| {
                routers_database(user_cfg, state.clone());
            }))
            .service(web::scope("/requests").configure(|user_cfg| {
                routers_request_type(user_cfg, state.clone());
                routers_request(user_cfg, state.clone());
            }))
            .service(web::scope("/education").configure(|user_cfg| {
                routers_education(user_cfg, state.clone());
            }))
            .service(web::scope("/school").configure(|user_cfg| {
                routers_sector(user_cfg, state.clone());
                routers_trade(user_cfg, state.clone());
                routers_school_auth(user_cfg, state.clone());
                routers_school(user_cfg, state.clone());
            }))
            .service(web::scope("/file").configure(|user_cfg| {
                routers_file_type(user_cfg, state.clone());
                routers_file(user_cfg, state.clone());
            }))
            .route("/auth/register", web::post().to(user_register_router))
            .route("/auth/login", web::post().to(user_login_router))
            .route("/auth/session", web::get().to(get_user_session_router))
            .route(
                "/auth/session",
                web::post().to(update_user_session_expires_router),
            )
            .route(
                "/auth/session/delete",
                web::post().to(delete_user_session_router),
            )
            .route(
                "/auth/oauth2/{provider}",
                web::get().to(oauth2_provider_url_router),
            ),
    );

    cfg.service(
        scope("/api/v0.0.2")
            .route("/", web::get().to(version_v002))
            .service(web::scope("/education").configure(|user_cfg| {
                education_routers(user_cfg, state.clone());
            }))
            .service(web::scope("/subject").configure(|user_cfg| {
                subject_routers(user_cfg, state.clone());
            }))
            .service(web::scope("/class").configure(|user_cfg| {
                main_class_router(user_cfg, state.clone());
            }))
            // trade
            .service(web::scope("").configure(|user_cfg| {
                routers_trade(user_cfg, state.clone());
            }))
            // sector
            .service(web::scope("").configure(|user_cfg| {
                routers_sector(user_cfg, state.clone());
            })),
    );

    cfg.service(
        web::resource("/ws") // WebSocket endpoint
            .route(web::get().to(websocket_handler)),
    );
}

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    ws::start(WebSocketConnection {}, &req, stream)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there! 🌼 this is space-together api version v0.0.1")
}
async fn version_v002() -> impl Responder {
    HttpResponse::Ok().body("Hey there! 🌼 this is space-together api version v0.0.2")
}

async fn welcome() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Welcome to the space-together API! We are happy to have you here. 😁💐🌼🌻",
        "description": "Space-Together is a school management system designed to help students, teachers, and school staff communicate and collaborate effectively. It works both online and offline, providing access to notes, books, homework, and notifications and other activities happen in school.",
        "routes": {
            "versions" : [
                {
                    "version": "v0.0.1",
                    "description": "This is the first version of the API, which includes basic functionality for user authentication, school management, and class management.",
                    "url" :"/api/v0.0.1"
                },
                {
                    "version": "v0.0.2",
                    "description": "This is the second version of the API, which includes additional features and improvements based on user feedback.",
                    "url" :"/api/v0.0.2"
                }
            ]
        }
    }))
}
