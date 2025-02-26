use std::sync::Arc;

use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use mongodb::{
    bson::{doc, to_document},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::{
        functions::{
            characters_fn::is_valid_username, data_type_fn::convert_fields_to_string,
            object_id::change_string_into_object_id,
        },
        schemas::education_schema::EducationSchema,
    },
    models::request_error_model::RequestErrorModel,
    AppState,
};

pub async fn get_educ_by_username(
    state: Arc<AppState>,
    username: &str,
) -> DbClassResult<EducationSchema> {
    match state
        .db
        .educations
        .collection
        .find_one(doc! {"username" : username})
        .await
    {
        Ok(Some(doc)) => Ok(doc),
        Ok(None) => Err(DbClassError::OtherError {
            err: "education username not found ,please use other username".to_string(),
        }),
        Err(_) => Err(DbClassError::OtherError {
            err: "Some thing went wrong to get education name by username".to_string(),
        }),
    }
}

pub async fn create_education_service(
    state: Data<AppState>,
    item: Json<EducationSchema>,
) -> impl Responder {
    match is_valid_username(&item.username.clone()).map_err(|err| DbClassError::OtherError {
        err: err.to_string(),
    }) {
        Err(e) => {
            return HttpResponse::BadRequest().json(RequestErrorModel {
                error: "Can't create new education because username is ready exit".to_string(),
                message: Some(e.to_string()),
            })
        }
        Ok(u) => {
            if let Ok(education) = get_educ_by_username(state.clone().into_inner(), &u).await {
                return HttpResponse::BadRequest().json(RequestErrorModel {
                    error: "Can't create new education because username is ready exit".to_string(),
                    message: Some(format!(
                        "education username is ready exit [{}], try other username for main class",
                        education.username
                    )),
                });
            }
        }
    }

    let index = IndexModel::builder()
        .keys(doc! {
        "username" : 1,
        })
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(err) = state.db.education.collection.create_index(index).await {
        return HttpResponse::BadRequest().json(RequestErrorModel {
            error: err.to_string(),
            message: Some(
                "Can't create education bcs username is leady exit , try other username"
                    .to_string(),
            ),
        });
    }

    match state
        .db
        .educations
        .create_new(item.into_inner(), "education")
        .await
    {
        Ok(doc) => {
            HttpResponse::Created().json(convert_fields_to_string(to_document(&doc).unwrap()))
        }
        Err(err) => HttpResponse::BadRequest().json(RequestErrorModel {
            error: err.to_string(),
            message: Some("Something went wrong to create new education".to_string()),
        }),
    }
}

pub async fn get_all_education_service(state: Data<AppState>) -> impl Responder {
    match state
        .db
        .educations
        .get_many(None, Some("education".to_string()))
        .await
    {
        Err(e) => HttpResponse::BadRequest().json(RequestErrorModel {
            error: e.to_string(),
            message: Some("Something went wrong to get education".to_string()),
        }),
        Ok(docs) => {
            let mut educations = Vec::new();
            for education in docs {
                educations.push(convert_fields_to_string(to_document(&education).unwrap()));
            }
            HttpResponse::Ok().json(educations)
        }
    }
}

pub async fn get_education_by_id_service(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match state
            .db
            .education
            .get_one_by_id(i, Some("education".to_string()))
            .await
        {
            Ok(doc) => {
                HttpResponse::Ok().json(convert_fields_to_string(to_document(&doc).unwrap()))
            }
            Err(e) => HttpResponse::BadRequest().json(RequestErrorModel {
                error: e.to_string(),
                message: Some("Something went wrong to get education".to_string()),
            }),
        },
    }
}

pub async fn get_education_by_username_service(
    state: Data<AppState>,
    username: Path<String>,
) -> impl Responder {
    match state
        .db
        .education
        .collection
        .find_one(doc! {"username" : &username.into_inner()})
        .await
    {
        Ok(doc) => HttpResponse::Ok().json(convert_fields_to_string(to_document(&doc).unwrap())),
        Err(e) => HttpResponse::BadRequest().json(RequestErrorModel {
            error: e.to_string(),
            message: Some("Something went wrong to get education".to_string()),
        }),
    }
}
