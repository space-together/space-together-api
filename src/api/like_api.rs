use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        like::{Like, LikePartial},
    },
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, like_service::LikeService},
    utils::request_context::{postgres_pool, request_context},
};

fn scoped_school_id(req: &HttpRequest, query: &RequestQuery) -> Option<String> {
    query
        .school_id
        .clone()
        .or_else(|| request_context(req).school_id)
}

#[get("")]
async fn get_all_likes(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = LikeService::new(postgres_pool(&state));
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
async fn get_all_likes_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = LikeService::new(postgres_pool(&state));
    let school_id = scoped_school_id(&req, &query);

    match service
        .get_all_with_relations(query.limit, query.skip, Some(&query), school_id.as_deref())
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_like_by_id(
    _req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = LikeService::new(postgres_pool(&state));

    match service.find_one(Some(&id), None, None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_like_by_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = LikeService::new(postgres_pool(&state));
    let school_id = scoped_school_id(&req, &query);

    match service
        .find_one(None, Some(&query), school_id.as_deref())
        .await
    {
        Ok(like) => HttpResponse::Ok().json(like),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/others/match")]
async fn get_like_by_other_match(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let service = LikeService::new(postgres_pool(&state));
    let school_id = scoped_school_id(&req, &query);

    match service
        .find_one_with_relations(None, Some(&query), school_id.as_deref())
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}/others")]
async fn get_like_by_id_with_relations(
    _req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = LikeService::new(postgres_pool(&state));

    match service.find_one_with_relations(Some(&id), None, None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_like(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Like>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let service = LikeService::new(postgres_pool(&state));
    let school_id = request_context(&req).school_id;

    match service.create(data.into_inner(), school_id).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "like",
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
async fn update_like(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<LikePartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let service = LikeService::new(postgres_pool(&state));

    match service.update(&id, &data.into_inner()).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "like",
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
async fn delete_like(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let service = LikeService::new(postgres_pool(&state));

    match service.delete(&id).await {
        Ok(like) => {
            let cloned = like.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "like",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &cloned,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(like)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_likes(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = LikeService::new(postgres_pool(&state));
    let school_id = scoped_school_id(&req, &query);

    match service
        .count_likes(query.filter.clone(), Some(&query), school_id.as_deref())
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_likes)
        .service(get_all_likes_with_relations)
        .service(count_likes)
        .service(get_like_by_match)
        .service(get_like_by_other_match)
        .service(get_like_by_id_with_relations)
        .service(get_like_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_like)
                .service(update_like)
                .service(delete_like),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "likes", blueprint);
}
