use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        main_class::{MainClass, MainClassPartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, main_class_service::MainClassService},
    utils::request_context::postgres_pool,
};

#[get("")]
async fn get_all_main_classes(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(postgres_pool(&state));
    match service.get_all(query.filter.clone(), query.limit, query.skip, Some(&query)).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/others")]
async fn get_all_main_classes_with_relations(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(postgres_pool(&state));
    match service
        .get_all_with_relations(query.filter.clone(), query.limit, query.skip, Some(&query))
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/others/match")]
async fn get_all_main_classes_by_other_match(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(postgres_pool(&state));
    match service.find_one_with_relations(None, Some(&query)).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}")]
async fn get_main_class_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = MainClassService::new(postgres_pool(&state));
    match service.find_one(Some(&id), None).await {
        Ok(main_class) => HttpResponse::Ok().json(main_class),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}/others")]
async fn get_main_class_by_id_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = MainClassService::new(postgres_pool(&state));
    match service.find_one_with_relations(Some(&id), None).await {
        Ok(main_class) => HttpResponse::Ok().json(main_class),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_main_class_by_match(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(postgres_pool(&state));
    match service.find_one(None, Some(&query)).await {
        Ok(main_class) => HttpResponse::Ok().json(main_class),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_main_class(
    _user: web::ReqData<AuthUserDto>,
    data: web::Json<MainClass>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(postgres_pool(&state));
    match service.create(data.into_inner()).await {
        Ok(main_class) => {
            let clone = main_class.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_created(
                        &state_clone, "main_class", &id.to_hex(), None, &clone,
                    ).await;
                }
            });
            HttpResponse::Created().json(main_class)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_main_class(
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<MainClassPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = MainClassService::new(postgres_pool(&state));
    match service.update(&id, &data.into_inner()).await {
        Ok(main_class) => {
            let clone = main_class.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_updated(
                        &state_clone, "main_class", &id.to_hex(), None, &clone,
                    ).await;
                }
            });
            HttpResponse::Ok().json(main_class)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_main_class(
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = MainClassService::new(postgres_pool(&state));
    match service.delete(&id).await {
        Ok(main_class) => {
            let clone = main_class.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_deleted(
                        &state_clone, "main_class", &id.to_hex(), None, &clone,
                    ).await;
                }
            });
            HttpResponse::Ok().json(main_class)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_main_classes(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(postgres_pool(&state));
    match service.count(query.filter.clone(), Some(&query)).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/main-classes")
            .service(get_all_main_classes)
            .service(get_all_main_classes_with_relations)
            .service(get_main_class_by_match)
            .service(get_all_main_classes_by_other_match)
            .service(count_main_classes)
            .service(get_main_class_by_id)
            .service(get_main_class_by_id_with_relations)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_main_class)
            .service(update_main_class)
            .service(delete_main_class),
    );
}
