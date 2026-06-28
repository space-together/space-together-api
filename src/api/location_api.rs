use actix_web::{get, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::location::{CellQuery, DistrictQuery, ProvinceQuery, SectorQuery, VillageQuery},
    services::location_service::LocationService,
    utils::request_context::postgres_pool,
};

#[get("/provinces")]
async fn get_provinces(
    query: web::Query<ProvinceQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = LocationService::new(postgres_pool(&state));
    match service.provinces(query.country.as_deref()).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/districts")]
async fn get_districts(
    query: web::Query<DistrictQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = LocationService::new(postgres_pool(&state));
    match service
        .districts(query.country.as_deref(), &query.province)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/sectors")]
async fn get_sectors(query: web::Query<SectorQuery>, state: web::Data<AppState>) -> impl Responder {
    let service = LocationService::new(postgres_pool(&state));
    match service
        .sectors(query.country.as_deref(), &query.province, &query.district)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/cells")]
async fn get_cells(query: web::Query<CellQuery>, state: web::Data<AppState>) -> impl Responder {
    let service = LocationService::new(postgres_pool(&state));
    match service
        .cells(
            query.country.as_deref(),
            &query.province,
            &query.district,
            &query.sector,
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/villages")]
async fn get_villages(
    query: web::Query<VillageQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = LocationService::new(postgres_pool(&state));
    match service
        .villages(
            query.country.as_deref(),
            &query.province,
            &query.district,
            &query.sector,
            &query.cell,
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/locations")
            .service(get_provinces)
            .service(get_districts)
            .service(get_sectors)
            .service(get_cells)
            .service(get_villages),
    );
}
