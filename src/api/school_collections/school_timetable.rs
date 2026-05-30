use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::school_timetable::{SchoolTimetable, SchoolTimetablePartial},
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType, school_token_model::SchoolToken},
    services::{
        education_year_service::{EducationYearQuery, EducationYearService},
        event_service::EventService,
        school_timetable_service::SchoolTimetableService,
    },
    utils::{object_id::ObjectId, request_context::postgres_pool},
};

#[get("")]
async fn get_all_timetables(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
    _req: HttpRequest,
) -> impl Responder {
    let service = SchoolTimetableService::new(postgres_pool(&state));
    match service.get_all(query.filter.clone(), query.limit, query.skip).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_timetable_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
    _req: HttpRequest,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SchoolTimetableService::new(postgres_pool(&state));
    match service.find_one(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/flied")]
async fn get_timetable_by_flied(
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
    req: HttpRequest,
) -> impl Responder {
    let school_id = req.extensions().get::<SchoolToken>().map(|c| c.id.clone());
    let service = SchoolTimetableService::new(postgres_pool(&state));

    let sid = query.school_id.as_deref().or(school_id.as_deref());
    match service.find_one(None, sid).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/school/{school_id}/year/{academic_year_id}")]
async fn get_by_school_and_academic_year(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
    _req: HttpRequest,
) -> impl Responder {
    let service = SchoolTimetableService::new(postgres_pool(&state));
    let (school_id_str, year_id_str) = path.into_inner();
    let school_id = IdType::to_object_id(&IdType::from_string(school_id_str));
    let academic_year_id = IdType::to_object_id(&IdType::from_string(year_id_str));

    if let (Ok(sid), Ok(ayid)) = (school_id, academic_year_id) {
        match service.find_by_school_and_academic_year(&sid, &Some(ayid)).await {
            Ok(data) => HttpResponse::Ok().json(data),
            Err(err) => HttpResponse::NotFound().json(err),
        }
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({ "message": "Invalid IDs provided" }))
    }
}

#[get("/current")]
async fn get_current_timetable(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let school_claims = match req.extensions().get::<SchoolToken>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({ "message": "School token required" }));
        }
    };

    let service = SchoolTimetableService::new(postgres_pool(&state));
    let education_year_service = EducationYearService::new(&state.pg.pool);

    let (education_year, _term_info) = match education_year_service
        .get_current_year_and_term(None, Some(EducationYearQuery::from_school_context(Some(school_claims.id.clone()))))
        .await
    {
        Ok(v) => v,
        Err(e) => return HttpResponse::BadRequest().json(e),
    };

    let school_id = match ObjectId::from_str(&school_claims.id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json("Invalid school ID"),
    };

    match service.find_by_school_and_academic_year(&school_id, &education_year.id).await {
        Ok(timetable) => HttpResponse::Ok().json(timetable),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_timetable(
    data: web::Json<SchoolTimetable>,
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let service = SchoolTimetableService::new(postgres_pool(&state));
    match service.create(data.into_inner()).await {
        Ok(tt) => {
            let clone = tt.clone();
            let state_clone = state.clone();
            let school_id_opt = get_school_id_from_request(&req);
            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_created(&state_clone, "school_timetable", &id.to_hex(), school_id_opt, &clone).await;
                }
            });
            HttpResponse::Created().json(tt)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/generate")]
async fn generate_timetable(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let school_claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({ "message": "School token required" })));
        }
    };

    let service = SchoolTimetableService::new(postgres_pool(&state));
    let education_year_service = EducationYearService::new(&state.pg.pool);

    let (education_year, _term_info) = match education_year_service
        .get_current_year_and_term(None, Some(EducationYearQuery::from_school_context(Some(school_claims.id.clone()))))
        .await
    {
        Ok(v) => v,
        Err(e) => return Ok(HttpResponse::BadRequest().json(e)),
    };

    let education_year_id = IdType::from_object_id(education_year.id.unwrap());
    let school_id = IdType::from_string(school_claims.id);
    let school_id_opt = get_school_id_from_request(&req);

    match service.generate_default(&school_id, &education_year_id).await {
        Ok(timetable) => {
            let timetable_clone = timetable.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = timetable_clone.id {
                    EventService::broadcast_created(&state_clone, "class_timetable", &id.to_hex(), school_id_opt, &timetable_clone).await;
                }
            });
            Ok(HttpResponse::Created().json(timetable))
        }
        Err(message) => Ok(HttpResponse::BadRequest().json(message)),
    }
}

#[put("/{id}")]
async fn update_timetable(
    path: web::Path<String>,
    data: web::Json<SchoolTimetablePartial>,
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SchoolTimetableService::new(postgres_pool(&state));
    let school_id_opt = get_school_id_from_request(&req);
    match service.update_timetable(&id, &data.into_inner()).await {
        Ok(tt) => {
            let clone = tt.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_updated(&state_clone, "school_timetable", &id.to_hex(), school_id_opt, &clone).await;
                }
            });
            HttpResponse::Ok().json(tt)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_timetable(
    path: web::Path<String>,
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SchoolTimetableService::new(postgres_pool(&state));
    let school_id_opt = get_school_id_from_request(&req);
    let before = service.find_one(Some(&id), None).await.ok();

    match service.delete_timetable(&id).await {
        Ok(res) => {
            if let Some(old) = before {
                let state_clone = state.clone();
                let clone = old.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = clone.id {
                        EventService::broadcast_deleted(&state_clone, "school_timetable", &id.to_hex(), school_id_opt, &clone).await;
                    }
                });
            }
            HttpResponse::Ok().json(res)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/school/timetables")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            .service(get_timetable_by_flied)
            .service(get_all_timetables)
            .service(get_current_timetable)
            .service(get_timetable_by_id)
            .service(get_by_school_and_academic_year)
            .service(create_timetable)
            .service(generate_timetable)
            .service(update_timetable)
            .service(delete_timetable),
    );
}
