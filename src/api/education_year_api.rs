use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        education_year::{EducationYear, EducationYearPartial},
    },
    guards::role_guard::check_admin,
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{
        education_year_service::{EducationYearQuery, EducationYearService},
        event_service::EventService,
    },
    utils::request_context::{postgres_pool, request_context},
};

#[get("")]
async fn get_all_education_years(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = EducationYearService::new(postgres_pool(&state));
    let context = request_context(&req);
    let year_query = match EducationYearQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .get_all(query.filter.clone(), query.limit, query.skip, year_query)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/current")]
async fn get_current_education_years(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = EducationYearService::new(postgres_pool(&state));
    let context = request_context(&req);

    match service
        .get_current_year_and_term(
            None,
            Some(EducationYearQuery::from_school_context(context.school_id)),
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/others")]
async fn get_all_education_years_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = EducationYearService::new(postgres_pool(&state));
    let context = request_context(&req);
    let year_query = match EducationYearQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .get_all_with_relations(query.filter.clone(), query.limit, query.skip, year_query)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_education_year_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = EducationYearService::new(postgres_pool(&state));
    let context = request_context(&req);

    match service
        .find_one(
            Some(&id),
            Some(EducationYearQuery::from_school_context(context.school_id)),
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}/others")]
async fn get_education_year_by_id_with_relations(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = EducationYearService::new(postgres_pool(&state));
    let context = request_context(&req);

    match service
        .find_one_with_relations(
            Some(&id),
            Some(EducationYearQuery::from_school_context(context.school_id)),
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_education_year_by_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = EducationYearService::new(postgres_pool(&state));
    let context = request_context(&req);
    let year_query = match EducationYearQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service.find_one(None, year_query).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/others/match")]
async fn get_education_year_by_other_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = EducationYearService::new(postgres_pool(&state));
    let context = request_context(&req);
    let year_query = match EducationYearQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service.find_one_with_relations(None, year_query).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/count")]
async fn count_education_years(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = EducationYearService::new(postgres_pool(&state));
    let context = request_context(&req);
    let year_query = match EducationYearQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .count_education_years(query.filter.clone(), year_query)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("")]
async fn create_education_year(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<EducationYear>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(err) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let service = EducationYearService::new(postgres_pool(&state));
    let context = request_context(&req);
    let mut education_year = data.into_inner();

    if education_year.created_by.is_none() {
        let user_id = match IdType::from_string(&user.id).to_object_id() {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
        education_year.created_by = Some(user_id);
    }
    if education_year.school_id.is_none() {
        if let Some(school_id) = context.school_id.or_else(|| user.current_school_id.clone()) {
            education_year.school_id = match IdType::from_string(school_id).to_object_id() {
                Ok(id) => Some(id),
                Err(err) => return HttpResponse::BadRequest().json(err),
            };
        }
    }

    match service.create(education_year).await {
        Ok(item) => {
            let item_clone = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = item_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "education_year",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &item_clone,
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
async fn update_education_year(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<EducationYearPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(err) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = EducationYearService::new(postgres_pool(&state));

    match service.update(&id, &data.into_inner()).await {
        Ok(item) => {
            let item_clone = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = item_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "education_year",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &item_clone,
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
async fn delete_education_year(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(err) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = EducationYearService::new(postgres_pool(&state));
    let user_id = match IdType::from_string(&user.id).to_object_id() {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service.delete(&id, user_id).await {
        Ok(item) => {
            let item_clone = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = item_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "education_year",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &item_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(item)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/{id}/restore")]
async fn restore_education_year(
    _req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(err) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = EducationYearService::new(postgres_pool(&state));

    match service.restore(&id).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_education_years)
        .service(get_all_education_years_with_relations)
        .service(get_current_education_years)
        .service(get_education_year_by_match)
        .service(get_education_year_by_other_match)
        .service(count_education_years)
        .service(get_education_year_by_id_with_relations)
        .service(get_education_year_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_education_year)
                .service(update_education_year)
                .service(delete_education_year)
                .service(restore_education_year),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "education-years", blueprint);
}
