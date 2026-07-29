use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        common_details::UserRole,
        parent::{Parent, ParentPartial},
    },
    guards::role_guard::{check_admin_or_staff, require_parent_child_access},
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{
        event_service::EventService,
        parent_service::{ParentQuery, ParentService},
    },
    utils::request_context::{postgres_pool, request_context},
};

// =========================
// ADMIN/STAFF ENDPOINTS
// =========================

#[get("")]
async fn get_all_parents(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
    user: web::ReqData<AuthUserDto>,
) -> impl Responder {
    if let Err(e) = check_admin_or_staff(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    let service = ParentService::new(postgres_pool(&state));
    let context = request_context(&req);
    let parent_query = match ParentQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .get_all(query.filter.clone(), query.limit, query.skip, parent_query)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/others")]
async fn get_all_parents_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
    user: web::ReqData<AuthUserDto>,
) -> impl Responder {
    if let Err(e) = check_admin_or_staff(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    let service = ParentService::new(postgres_pool(&state));
    let context = request_context(&req);
    let parent_query = match ParentQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .get_all_with_relations(query.filter.clone(), query.limit, query.skip, parent_query)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_parent_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
    user: web::ReqData<AuthUserDto>,
) -> impl Responder {
    if let Err(e) = check_admin_or_staff(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ParentService::new(postgres_pool(&state));
    let context = request_context(&req);
    let parent_query = Some(ParentQuery::from_school_context(context.school_id));

    match service.find_one(Some(&id), parent_query).await {
        Ok(parent) => HttpResponse::Ok().json(parent),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}/others")]
async fn get_parent_by_id_with_relations(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
    user: web::ReqData<AuthUserDto>,
) -> impl Responder {
    if let Err(e) = check_admin_or_staff(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ParentService::new(postgres_pool(&state));
    let context = request_context(&req);
    let parent_query = Some(ParentQuery::from_school_context(context.school_id));

    match service
        .find_one_with_relations(Some(&id), parent_query)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_parent(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Parent>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(e) = check_admin_or_staff(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    let service = ParentService::new(postgres_pool(&state));
    let context = request_context(&req);

    let mut parent = data.into_inner();
    if parent.school_id.is_none() {
        if let Some(school_id) = context.school_id.or_else(|| user.current_school_id.clone()) {
            parent.school_id = match IdType::from_string(school_id).to_object_id() {
                Ok(id) => Some(id),
                Err(err) => return HttpResponse::BadRequest().json(err),
            };
        }
    }

    match service.create(parent).await {
        Ok(parent) => {
            let parent_clone = parent.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = parent_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "parent",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &parent_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(parent)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_parent(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<ParentPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(e) = check_admin_or_staff(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ParentService::new(postgres_pool(&state));

    match service.update(&id, &data.into_inner()).await {
        Ok(parent) => {
            let parent_clone = parent.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = parent_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "parent",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &parent_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(parent)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_parent(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(e) = check_admin_or_staff(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ParentService::new(postgres_pool(&state));

    match service.delete(&id).await {
        Ok(parent) => {
            let parent_clone = parent.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = parent_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "parent",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &parent_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(parent)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_parents(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
    user: web::ReqData<AuthUserDto>,
) -> impl Responder {
    if let Err(e) = check_admin_or_staff(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    let service = ParentService::new(postgres_pool(&state));
    let context = request_context(&req);
    let parent_query = match ParentQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .count_parents(query.filter.clone(), parent_query)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

// =========================
// PARENT PORTAL ENDPOINTS
// =========================

#[get("/dashboard")]
async fn get_parent_dashboard(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check if user is a parent
    if user.role != Some(UserRole::PARENT) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({ "error": "Only parents can access this endpoint" }));
    }

    let school_id = match get_school_id_from_request(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "School ID required" }))
        }
    };

    let service = ParentService::new(postgres_pool(&state));

    let parent = match service.find_by_user_id(&user.id, Some(&school_id)).await {
        Ok(p) => p,
        Err(err) => return HttpResponse::NotFound().json(err),
    };

    let parent_id = match parent.id {
        Some(id) => IdType::ObjectId(id),
        None => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Parent ID not found" }))
        }
    };

    match service.get_dashboard(&parent_id, &school_id, &state).await {
        Ok(dashboard) => HttpResponse::Ok().json(dashboard),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{student_id}/attendance")]
async fn get_student_attendance(
    req: HttpRequest,
    path: web::Path<String>,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let student_id = path.into_inner();

    let school_id = match get_school_id_from_request(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "School ID required" }))
        }
    };

    let service = ParentService::new(postgres_pool(&state));

    // Validate parent-child access using new guard
    if let Err(e) = require_parent_child_access(&user, &student_id, &service).await {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    match service
        .get_attendance_summary(&student_id, &school_id, &state)
        .await
    {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{student_id}/results")]
async fn get_student_results(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<RequestQuery>,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let student_id = path.into_inner();

    let school_id = match get_school_id_from_request(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "School ID required" }))
        }
    };

    let service = ParentService::new(postgres_pool(&state));

    // Validate parent-child access using new guard
    if let Err(e) = require_parent_child_access(&user, &student_id, &service).await {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    let education_year_id = query.education_year_id.as_deref();
    let term_id = query.term_id.as_deref();

    match service
        .get_student_results(&student_id, &school_id, education_year_id, term_id, &state)
        .await
    {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{student_id}/finance")]
async fn get_student_finance(
    req: HttpRequest,
    path: web::Path<String>,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let student_id = path.into_inner();

    let school_id = match get_school_id_from_request(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "School ID required" }))
        }
    };

    let service = ParentService::new(postgres_pool(&state));

    // Validate parent-child access using new guard
    if let Err(e) = require_parent_child_access(&user, &student_id, &service).await {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }

    match service
        .get_finance_summary(&student_id, &school_id, &state)
        .await
    {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/announcements")]
async fn get_parent_announcements(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check if user is a parent
    if user.role != Some(UserRole::PARENT) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({ "error": "Only parents can access this endpoint" }));
    }

    let school_id = match get_school_id_from_request(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "School ID required" }))
        }
    };

    let service = ParentService::new(postgres_pool(&state));

    let parent = match service.find_by_user_id(&user.id, Some(&school_id)).await {
        Ok(p) => p,
        Err(err) => return HttpResponse::NotFound().json(err),
    };

    let parent_id = match parent.id {
        Some(id) => IdType::ObjectId(id),
        None => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Parent ID not found" }))
        }
    };

    match service
        .get_parent_announcements(&parent_id, &school_id, query.limit, query.skip, &state)
        .await
    {
        Ok(announcements) => HttpResponse::Ok().json(announcements),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            // Admin/Staff endpoints
            .service(get_all_parents)
            .service(get_all_parents_with_relations)
            .service(count_parents)
            .service(get_parent_by_id_with_relations)
            .service(get_parent_by_id)
            .service(create_parent)
            .service(update_parent)
            .service(delete_parent)
            // Parent portal endpoints
            .service(get_parent_dashboard)
            .service(get_student_attendance)
            .service(get_student_results)
            .service(get_student_finance)
            .service(get_parent_announcements),
    );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "parents", blueprint);
}
