use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::class_subject::{ClassSubject, ClassSubjectPartial},
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{
        class_subject_service::{ClassSubjectQuery, ClassSubjectService},
        event_service::EventService,
    },
    utils::request_context::{postgres_pool, request_context},
};

#[get("")]
async fn get_all_class_subjects(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ClassSubjectService::new(postgres_pool(&state));
    let context = request_context(&req);
    let subject_query = match ClassSubjectQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .get_all(query.filter.clone(), query.limit, query.skip, subject_query)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/others")]
async fn get_all_class_subjects_with_others(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ClassSubjectService::new(postgres_pool(&state));
    let context = request_context(&req);
    let subject_query = match ClassSubjectQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .get_all_with_relations(query.filter.clone(), query.limit, query.skip, subject_query)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_class_subject_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ClassSubjectService::new(postgres_pool(&state));
    let context = request_context(&req);
    let id = IdType::from_string(path.into_inner());

    match service
        .find_one(
            Some(&id),
            Some(ClassSubjectQuery::from_school_context(context.school_id)),
        )
        .await
    {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_class_subject_by_match(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let service = ClassSubjectService::new(postgres_pool(&state));
    let context = request_context(&req);
    let subject_query = match ClassSubjectQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service.find_one(None, subject_query).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}/others")]
async fn get_class_subject_by_id_others(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ClassSubjectService::new(postgres_pool(&state));
    let context = request_context(&req);
    let id = IdType::from_string(path.into_inner());

    match service
        .find_one_with_relations(
            Some(&id),
            Some(ClassSubjectQuery::from_school_context(context.school_id)),
        )
        .await
    {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/others/match")]
async fn get_class_subject_by_match_others(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let service = ClassSubjectService::new(postgres_pool(&state));
    let context = request_context(&req);
    let subject_query = match ClassSubjectQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service.find_one_with_relations(None, subject_query).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/count")]
async fn count_subjects(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ClassSubjectService::new(postgres_pool(&state));
    let context = request_context(&req);
    let subject_query = match ClassSubjectQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .count_subject(query.filter.clone(), subject_query)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("")]
async fn create_class_subject(
    data: web::Json<ClassSubject>,
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let service = ClassSubjectService::new(postgres_pool(&state));
    let context = request_context(&req);
    let mut subject = data.into_inner();
    if subject.school_id.is_none() {
        if let Some(school_id) = context.school_id {
            subject.school_id = match IdType::from_string(school_id).to_object_id() {
                Ok(id) => Some(id),
                Err(err) => return HttpResponse::BadRequest().json(err),
            };
        }
    }

    match service.create(subject).await {
        Ok(subject) => {
            let clone = subject.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "class_subject",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(subject)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_class_subject(
    path: web::Path<String>,
    data: web::Json<ClassSubjectPartial>,
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let service = ClassSubjectService::new(postgres_pool(&state));
    let id = IdType::from_string(path.into_inner());

    match service.update_subject(&id, &data.into_inner()).await {
        Ok(subject) => {
            let clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "class_subject",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(subject)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_class_subject(
    path: web::Path<String>,
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let service = ClassSubjectService::new(postgres_pool(&state));
    let id = IdType::from_string(path.into_inner());

    match service.delete_subject(&id).await {
        Ok(subject) => {
            let state_clone = state.clone();
            let clone = subject.clone();
            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "class_subject",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(subject)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_class_subjects)
        .service(get_all_class_subjects_with_others)
        .service(get_class_subject_by_match)
        .service(get_class_subject_by_match_others)
        .service(count_subjects)
        .service(get_class_subject_by_id)
        .service(get_class_subject_by_id_others)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_class_subject)
                .service(update_class_subject)
                .service(delete_class_subject),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "class-subjects", blueprint);
}
