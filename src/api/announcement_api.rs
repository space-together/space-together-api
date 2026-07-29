use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        announcement::{Announcement, AnnouncementPartial},
        auth_user::AuthUserDto,
    },
    guards::role_guard::check_admin_or_staff,
    handler::delete_target_handler::delete_target_handler,
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{announcement_service::AnnouncementService, event_service::EventService},
    utils::request_context::{postgres_pool, request_context},
};

fn scoped_school_id(req: &HttpRequest, query: &RequestQuery) -> Option<String> {
    query
        .school_id
        .clone()
        .or_else(|| request_context(req).school_id)
}

#[get("")]
async fn get_all_announcements(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = AnnouncementService::new(postgres_pool(&state));
    let school_id = scoped_school_id(&req, &query);

    match service
        .get_all(
            query.filter.clone(),
            query.limit,
            query.skip,
            Some(&query),
            school_id.as_deref(),
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/others")]
async fn get_all_announcements_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = AnnouncementService::new(postgres_pool(&state));
    let school_id = scoped_school_id(&req, &query);

    match service
        .get_all_with_relations(
            query.filter.clone(),
            query.limit,
            query.skip,
            Some(&query),
            school_id.as_deref(),
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}/others")]
async fn get_announcement_by_id_with_relations(
    _req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = AnnouncementService::new(postgres_pool(&state));

    match service.find_one_with_relations(Some(&id), None, None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}")]
async fn get_announcement_by_id(
    _req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = AnnouncementService::new(postgres_pool(&state));

    match service.find_one(Some(&id), None, None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_announcement_by_match(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let service = AnnouncementService::new(postgres_pool(&state));
    let school_id = scoped_school_id(&req, &query);

    match service
        .find_one(None, Some(&query), school_id.as_deref())
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/others/match")]
async fn get_announcement_by_other_match(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let service = AnnouncementService::new(postgres_pool(&state));
    let school_id = scoped_school_id(&req, &query);

    match service
        .find_one_with_relations(None, Some(&query), school_id.as_deref())
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_announcement(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Announcement>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(e) = check_admin_or_staff(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": e
        }));
    }

    let service = AnnouncementService::new(postgres_pool(&state));
    let school_id = request_context(&req).school_id;

    match service.create(data.into_inner(), school_id).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "announcement",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &cloned,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(item)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_announcement(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<AnnouncementPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(e) = check_admin_or_staff(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": e
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = AnnouncementService::new(postgres_pool(&state));

    match service.update(&id, &data.into_inner()).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "announcement",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &cloned,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(item)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_announcement(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let service = AnnouncementService::new(postgres_pool(&state));

    match service.delete(&id).await {
        Ok(announcement) => {
            if let Some(target_id) = announcement.id {
                match delete_target_handler(postgres_pool(&state), &target_id).await {
                    Ok(_) => {}
                    Err(err) => return HttpResponse::BadRequest().json(err),
                }
            }

            let cloned = announcement.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "announcement",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &cloned,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(announcement)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_announcements(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = AnnouncementService::new(postgres_pool(&state));
    let school_id = scoped_school_id(&req, &query);

    match service
        .count_announcements(query.filter.clone(), Some(&query), school_id.as_deref())
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_announcements)
        .service(get_all_announcements_with_relations)
        .service(get_announcement_by_match)
        .service(get_announcement_by_other_match)
        .service(count_announcements)
        .service(get_announcement_by_id)
        .service(get_announcement_by_id_with_relations)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_announcement)
                .service(update_announcement)
                .service(delete_announcement),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "announcements", blueprint);
}
