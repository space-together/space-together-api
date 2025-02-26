use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use mongodb::{
    bson::{doc, to_document},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    libs::{
        data::class_data::get_main_class_by_username,
        functions::data_type_fn::convert_fields_to_string,
        schemas::main_class_schema::ClassRoomSchema,
    },
    models::request_error_model::RequestErrorModel,
    AppState,
};
pub async fn create_main_class(
    state: Data<AppState>,
    item: Json<ClassRoomSchema>,
) -> impl Responder {
    if let Some(username) = &item.username.clone() {
        if get_main_class_by_username(state.clone().into_inner(), username)
            .await
            .is_ok()
        {
            return HttpResponse::BadRequest().json(RequestErrorModel {
                error: "Can't create new class room because username is ready exit".to_string(),
                message: Some(
                    "main class room username is ready exit, try other username for main class"
                        .to_string(),
                ),
            });
        }
    }

    let index = IndexModel::builder()
        .keys(doc! {
        "username" : 1,
        })
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(e) = state.db.subject.collection.create_index(index).await {
        return HttpResponse::BadRequest().json(RequestErrorModel {
            error: e.to_string(),
            message: Some(
                "main class room username is ready exit, try other username for main class"
                    .to_string(),
            ),
        });
    }

    match state
        .db
        .main_class
        .create_new(item.into_inner(), "class_room")
        .await
    {
        Err(e) => HttpResponse::BadRequest().json(RequestErrorModel {
            error: e.to_string(),
            message: Some("Can't create subject, try again later".to_string()),
        }),
        Ok(doc) => {
            HttpResponse::Created().json(convert_fields_to_string(to_document(&doc).unwrap()))
        }
    }
}

pub async fn get_all_main_class_room(state: Data<AppState>) -> impl Responder {
    match state
        .db
        .main_class
        .get_many(None, Some("main_class".to_string()))
        .await
    {
        Ok(docs) => {
            let mut all_main_class = Vec::new();
            for main_class in docs {
                all_main_class.push(convert_fields_to_string(to_document(&main_class).unwrap()));
            }
            HttpResponse::Ok().json(all_main_class)
        }
        Err(e) => HttpResponse::BadRequest().json(RequestErrorModel {
            error: e.to_string(),
            message: Some("Some thing went worn to get all main classes".to_string()),
        }),
    }
}
