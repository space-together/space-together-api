use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        exam::{Exam, ExamPartial},
    },
    guards::role_guard::check_admin_staff_or_teacher,
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{
        event_service::EventService,
        exam_service::{ExamQuery, ExamService},
    },
    utils::request_context::{postgres_pool, request_context},
};

#[get("")]
async fn get_all_exams(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ExamService::new(postgres_pool(&state));
    let context = request_context(&req);
    let exam_query = match ExamQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .get_all(query.filter.clone(), query.limit, query.skip, exam_query)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_exam_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = ExamService::new(postgres_pool(&state));
    let context = request_context(&req);

    match service
        .find_one(&id, Some(ExamQuery::from_school_context(context.school_id)))
        .await
    {
        Ok(exam) => HttpResponse::Ok().json(exam),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_exam(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Exam>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(err) = check_admin_staff_or_teacher(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err }));
    }

    let service = ExamService::new(postgres_pool(&state));
    let context = request_context(&req);
    let mut exam = data.clone();

    if exam.created_by.is_none() {
        let user_id = match IdType::from_string(&user.id).to_object_id() {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
        exam.created_by = Some(user_id);
    }
    if exam.school_id.is_none() {
        if let Some(school_id) = context.school_id.or_else(|| user.current_school_id.clone()) {
            exam.school_id = match IdType::from_string(school_id).to_object_id() {
                Ok(id) => Some(id),
                Err(err) => return HttpResponse::BadRequest().json(err),
            };
        }
    }

    match service.create(exam).await {
        Ok(exam) => {
            let exam_clone = exam.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = exam_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "exam",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &exam_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(exam)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_exam(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<ExamPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(err) = check_admin_staff_or_teacher(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ExamService::new(postgres_pool(&state));

    match service.update(&id, &data.into_inner()).await {
        Ok(exam) => {
            let exam_clone = exam.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = exam_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "exam",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &exam_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(exam)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_exam(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(err) = check_admin_staff_or_teacher(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ExamService::new(postgres_pool(&state));

    match service.delete(&id).await {
        Ok(exam) => {
            let exam_clone = exam.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = exam_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "exam",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &exam_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(exam)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/{id}/publish")]
async fn publish_exam(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = ExamService::new(postgres_pool(&state));

    match service.publish(&id).await {
        Ok(exam) => {
            let exam_clone = exam.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = exam_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "exam",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &exam_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(exam)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_exams(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ExamService::new(postgres_pool(&state));
    let context = request_context(&req);
    let exam_query = match ExamQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service.count_exams(query.filter.clone(), exam_query).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_exams)
        .service(count_exams)
        .service(get_exam_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_exam)
                .service(update_exam)
                .service(delete_exam)
                .service(publish_exam),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "exams", blueprint);
}
