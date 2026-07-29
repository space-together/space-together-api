use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        sector::{Sector, SectorPartial},
    },
    models::{
        api_request_model::{GetByIdsBody, RequestQuery},
        id_model::IdType,
    },
    services::{event_service::EventService, sector_service::SectorService},
    utils::request_context::postgres_pool,
};

#[get("")]
async fn get_all_sectors(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SectorService::new(postgres_pool(&state));
    match service.get_all(query.filter.clone(), query.limit, query.skip, Some(&query)).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_sector_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SectorService::new(postgres_pool(&state));
    match service.find_one(Some(&id), None).await {
        Ok(sector) => HttpResponse::Ok().json(sector),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_sector_by_match(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SectorService::new(postgres_pool(&state));
    match service.find_one(None, Some(&query)).await {
        Ok(sector) => HttpResponse::Ok().json(sector),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_sector(
    _user: web::ReqData<AuthUserDto>,
    data: web::Json<Sector>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SectorService::new(postgres_pool(&state));
    match service.create(data.into_inner()).await {
        Ok(sector) => {
            let sector_clone = sector.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = sector_clone.id {
                    EventService::broadcast_created(
                        &state_clone, "sector", &id.to_hex(), None, &sector_clone,
                    ).await;
                }
            });
            HttpResponse::Created().json(sector)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_sector(
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<SectorPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SectorService::new(postgres_pool(&state));
    match service.update(&id, &data.into_inner()).await {
        Ok(sector) => {
            let sector_clone = sector.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = sector_clone.id {
                    EventService::broadcast_updated(
                        &state_clone, "sector", &id.to_hex(), None, &sector_clone,
                    ).await;
                }
            });
            HttpResponse::Ok().json(sector)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_sector(
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SectorService::new(postgres_pool(&state));
    match service.delete(&id).await {
        Ok(sector) => {
            let sector_clone = sector.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = sector_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone, "sector", &id.to_hex(), None, &sector_clone,
                    ).await;
                }
            });
            HttpResponse::Ok().json(sector)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_sectors(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SectorService::new(postgres_pool(&state));
    match service.count(query.filter.clone(), Some(&query)).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/by-ids")]
async fn get_sectors_by_ids(
    body: web::Json<GetByIdsBody>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SectorService::new(postgres_pool(&state));
    let ids: Vec<IdType> = body.ids.iter().map(|id| IdType::from_string(id.clone())).collect();
    match service.find_by_ids(ids).await {
        Ok(sectors) => HttpResponse::Ok().json(sectors),
        Err(error) => HttpResponse::NotFound().json(error),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sectors")
            .service(get_all_sectors)
            .service(get_sector_by_match)
            .service(count_sectors)
            .service(get_sectors_by_ids)
            .service(get_sector_by_id)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_sector)
            .service(update_sector)
            .service(delete_sector),
    );
}
