use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        grading_scale::{GradingScale, GradingScalePartial},
    },
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{
        event_service::EventService,
        grading_scale_service::{GradingScaleQuery, GradingScaleService},
    },
    utils::{
        object_id::parse_object_id_value,
        request_context::{postgres_pool, request_context},
    },
};

#[get("")]
async fn get_all_grading_scales(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = GradingScaleService::new(postgres_pool(&state));
    let context = request_context(&req);
    let scale_query = match GradingScaleQuery::from_request(&query, context.school_id) {
        Ok(query) => Some(query),
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .get_all(query.filter.clone(), query.limit, query.skip, scale_query)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_grading_scale_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = GradingScaleService::new(postgres_pool(&state));
    let context = request_context(&req);

    match service
        .find_one(
            &id,
            Some(GradingScaleQuery::from_school_context(context.school_id)),
        )
        .await
    {
        Ok(scale) => HttpResponse::Ok().json(scale),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_grading_scale(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<GradingScale>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = GradingScaleService::new(postgres_pool(&state));
    let context = request_context(&req);

    let mut scale = data.clone();

    if scale.created_by.is_none() {
        let user_id = match parse_object_id_value(&user.id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
        scale.created_by = Some(user_id);
    }
    if scale.school_id.is_none() {
        if let Some(school_id) = context.school_id.or_else(|| user.current_school_id.clone()) {
            scale.school_id = match IdType::from_string(school_id).to_object_id() {
                Ok(id) => Some(id),
                Err(err) => return HttpResponse::BadRequest().json(err),
            };
        }
    }

    match service.create(scale).await {
        Ok(scale) => {
            let scale_clone = scale.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = scale_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "grading_scale",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &scale_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(scale)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_grading_scale(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<GradingScalePartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = GradingScaleService::new(postgres_pool(&state));

    match service.update(&id, &data.into_inner()).await {
        Ok(scale) => {
            let scale_clone = scale.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = scale_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "grading_scale",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &scale_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(scale)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/{id}/activate")]
async fn activate_grading_scale(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = GradingScaleService::new(postgres_pool(&state));

    match service.activate(&id).await {
        Ok(scale) => {
            let scale_clone = scale.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = scale_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "grading_scale",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &scale_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(scale)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_grading_scales)
        .service(get_grading_scale_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_grading_scale)
                .service(update_grading_scale)
                .service(activate_grading_scale),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "grading-scales", blueprint);
}
