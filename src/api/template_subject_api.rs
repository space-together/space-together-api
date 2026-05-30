use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        template_subject::{TemplateSubject, TemplateSubjectPartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, template_subject_service::TemplateSubjectService},
    utils::request_context::postgres_pool,
};

#[get("")]
async fn get_all_template_subjects(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.get_all(query.filter.clone(), query.limit, query.skip, Some(&query)).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

#[get("/others")]
async fn get_all_template_subjects_with_others(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.get_all_with_other(query.filter.clone(), query.limit, query.skip).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

#[get("/{id}")]
async fn get_template_subject_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.find_one_by_id(&id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

#[get("/{id}/others")]
async fn get_template_subject_by_id_others(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.find_one_with_relations(&id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

#[get("/code/{code}")]
async fn get_template_subject_by_code(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let code = path.into_inner();
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.find_one_by_code(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

#[get("/code/{code}/others")]
async fn get_template_subject_by_code_others(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let code = path.into_inner();
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.find_one_with_relations_by_code(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

#[get("/prerequisite/{id}/others")]
async fn find_many_by_prerequisite_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.find_many_by_prerequisite_with_relations(&id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

#[get("/prerequisite/{id}")]
async fn find_many_by_prerequisite(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.find_many_by_prerequisite(&id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

#[post("")]
async fn create_template_subject(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<TemplateSubject>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.create(data.into_inner()).await {
        Ok(subject) => {
            let subject_clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_created(&state_clone, "template_subject", &id.to_hex(), None, &subject_clone).await;
                }
            });
            HttpResponse::Created().json(subject)
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

#[put("/{id}")]
async fn update_template_subject(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<TemplateSubjectPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.update_subject(&id, &data.into_inner()).await {
        Ok(subject) => {
            let subject_clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_updated(&state_clone, "template_subject", &id.to_hex(), None, &subject_clone).await;
                }
            });
            HttpResponse::Ok().json(subject)
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

#[delete("/{id}")]
async fn delete_template_subject(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.delete_subject(&id).await {
        Ok(subject) => {
            let subject_clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_deleted(&state_clone, "template_subject", &id.to_hex(), None, &subject_clone).await;
                }
            });
            HttpResponse::Ok().json(serde_json::json!(subject))
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

#[get("/count")]
async fn count_template_subjects(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = TemplateSubjectService::new(postgres_pool(&state));
    match service.count_template_subjects(query.filter.clone(), Some(&query)).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/template-subjects")
            .service(get_all_template_subjects)
            .service(get_all_template_subjects_with_others)
            .service(get_template_subject_by_code)
            .service(get_template_subject_by_code_others)
            .service(find_many_by_prerequisite_with_relations)
            .service(find_many_by_prerequisite)
            .service(count_template_subjects)
            .service(get_template_subject_by_id)
            .service(get_template_subject_by_id_others)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_template_subject)
            .service(update_template_subject)
            .service(delete_template_subject),
    );
}
