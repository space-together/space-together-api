use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        comment::{Comment, CommentPartial},
    },
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{comment_service::CommentService, event_service::EventService},
    utils::request_context::{postgres_pool, request_context},
};

fn scoped_school_id(req: &HttpRequest, query: &RequestQuery) -> Option<String> {
    query
        .school_id
        .clone()
        .or_else(|| request_context(req).school_id)
}

#[get("")]
async fn get_all_comments(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = CommentService::new(postgres_pool(&state));
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
async fn get_all_comments_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = CommentService::new(postgres_pool(&state));
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

#[get("/{id}")]
async fn get_comment_by_id(
    _req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = CommentService::new(postgres_pool(&state));

    match service.find_one(Some(&id), None, None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_comment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Comment>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let service = CommentService::new(postgres_pool(&state));
    let school_id = request_context(&req).school_id;

    match service.create(data.into_inner(), school_id).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "comment",
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
async fn update_comment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<CommentPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let service = CommentService::new(postgres_pool(&state));

    match service.update(&id, &data.into_inner()).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "comment",
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
async fn delete_comment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let service = CommentService::new(postgres_pool(&state));

    match service.delete(&id).await {
        Ok(comment) => {
            let cloned = comment.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "comment",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &cloned,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(comment)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_comments(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = CommentService::new(postgres_pool(&state));
    let school_id = scoped_school_id(&req, &query);

    match service
        .count_comments(query.filter.clone(), Some(&query), school_id.as_deref())
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_comments)
        .service(get_all_comments_with_relations)
        .service(count_comments)
        .service(get_comment_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_comment)
                .service(update_comment)
                .service(delete_comment),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "comments", blueprint);
}
