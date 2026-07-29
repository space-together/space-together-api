use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        school_staff::{SchoolStaff, SchoolStaffPartial},
    },
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{
        event_service::EventService,
        school_staff_service::{SchoolStaffQuery, SchoolStaffService},
    },
    utils::request_context::{postgres_pool, request_context},
};

#[get("")]
async fn get_all_school_staff(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SchoolStaffService::new(postgres_pool(&state));
    let context = request_context(&req);
    let staff_query = match SchoolStaffQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .get_all(query.filter.clone(), query.limit, query.skip, staff_query)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_school_staff_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SchoolStaffService::new(postgres_pool(&state));
    let context = request_context(&req);
    let staff_query = Some(SchoolStaffQuery::from_school_context(context.school_id));

    match service.find_one(Some(&id), staff_query).await {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_school_staff_by_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SchoolStaffService::new(postgres_pool(&state));
    let context = request_context(&req);
    let staff_query = match SchoolStaffQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service.find_one(None, staff_query).await {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_school_staff(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<SchoolStaff>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SchoolStaffService::new(postgres_pool(&state));
    let context = request_context(&req);

    let mut staff = data.clone();

    if staff.creator_id.is_none() {
        let user_id = match IdType::from_string(&user.id).to_object_id() {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
        staff.creator_id = Some(user_id);
    }
    if staff.school_id.is_none() {
        if let Some(school_id) = context.school_id.or_else(|| user.current_school_id.clone()) {
            staff.school_id = match IdType::from_string(school_id).to_object_id() {
                Ok(id) => Some(id),
                Err(err) => return HttpResponse::BadRequest().json(err),
            };
        }
    }

    match service.create(staff, Some(&state)).await {
        Ok(staff) => {
            let staff_clone = staff.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = staff_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "school_staff",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &staff_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(staff)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_school_staff(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<SchoolStaffPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SchoolStaffService::new(postgres_pool(&state));

    match service.update(&id, &data.into_inner()).await {
        Ok(staff) => {
            let staff_clone = staff.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = staff_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "school_staff",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &staff_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(staff)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_school_staff(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SchoolStaffService::new(postgres_pool(&state));

    match service.delete(&id).await {
        Ok(staff) => {
            let staff_clone = staff.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = staff_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "school_staff",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &staff_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(staff)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_school_staff(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SchoolStaffService::new(postgres_pool(&state));
    let context = request_context(&req);
    let staff_query = match SchoolStaffQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service.count_staff(query.filter.clone(), staff_query).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_school_staff)
        .service(get_school_staff_by_match)
        .service(count_school_staff)
        .service(get_school_staff_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_school_staff)
                .service(update_school_staff)
                .service(delete_school_staff),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "school-staff", blueprint);
}
