use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        class_timetable::{ClassTimetable, ClassTimetablePartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{class_timetable_service::ClassTimetableService, event_service::EventService},
    utils::request_context::postgres_pool,
};

#[get("")]
async fn get_all_timetables(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ClassTimetableService::new(postgres_pool(&state));
    match service.get_all(query.filter.clone(), query.limit, query.skip).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

#[get("/{id}")]
async fn get_timetable_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = ClassTimetableService::new(postgres_pool(&state));
    match service.find_one_by_id(&id).await {
        Ok(timetable) => HttpResponse::Ok().json(timetable),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

#[post("")]
async fn create_timetable(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<ClassTimetable>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let service = ClassTimetableService::new(postgres_pool(&state));
    match service.create(data.into_inner()).await {
        Ok(timetable) => {
            let timetable_clone = timetable.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = timetable_clone.id {
                    EventService::broadcast_created(&state_clone, "class_timetable", &id.to_hex(), None, &timetable_clone).await;
                }
            });
            HttpResponse::Created().json(timetable)
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

#[put("/{id}")]
async fn update_timetable(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<ClassTimetablePartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ClassTimetableService::new(postgres_pool(&state));
    match service.update_timetable(&id, &data.into_inner()).await {
        Ok(timetable) => {
            let timetable_clone = timetable.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = timetable_clone.id {
                    EventService::broadcast_updated(&state_clone, "class_timetable", &id.to_hex(), None, &timetable_clone).await;
                }
            });
            HttpResponse::Ok().json(timetable)
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

#[delete("/{id}")]
async fn delete_timetable(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ClassTimetableService::new(postgres_pool(&state));
    let before_delete = service.find_one_by_id(&id).await.ok();

    match service.delete_timetable(&id).await {
        Ok(_) => {
            if let Some(timetable) = before_delete {
                let timetable_clone = timetable.clone();
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = timetable_clone.id {
                        EventService::broadcast_deleted(&state_clone, "class_timetable", &id.to_hex(), None, &timetable_clone).await;
                    }
                });
            }
            HttpResponse::Ok().json(serde_json::json!({ "message": "Class timetable deleted successfully" }))
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/class-timetables")
            .service(get_all_timetables)
            .service(get_timetable_by_id)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_timetable)
            .service(update_timetable)
            .service(delete_timetable),
    );
}
