use crate::{
    controllers::class_controller::class_room_controller::get_class_room_by_id,
    libs::{
        functions::data_type_fn::convert_fields_to_string, schemas::subject_schema::SubjectSchema,
        types::fields_types::IdType,
    },
    models::request_error_model::RequestErrorModel,
    AppState,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use mongodb::{
    bson::{doc, to_document},
    options::IndexOptions,
    IndexModel,
};

pub async fn create_subject_servicer(
    state: Data<AppState>,
    item: Json<SubjectSchema>,
) -> impl Responder {
    let index = IndexModel::builder()
        .keys(doc! {
        "code" : 1,
        })
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(e) = state.db.subject.collection.create_index(index).await {
        return HttpResponse::BadRequest().json(RequestErrorModel {
            error: e.to_string(),
            message: Some("Subject Code is ready exit, try other code".to_string()),
        });
    }

    if let Some(main_class) = item.class_room_id.clone() {
        if get_class_room_by_id(
            state.clone().into_inner(),
            IdType::to_object(&main_class).unwrap(),
        )
        .await
        .is_err()
        {
            return HttpResponse::BadRequest().json(RequestErrorModel {
                message: Some(format!(
                    "Main class id is not exit [{}], try other main class id",
                    IdType::to_string(&main_class)
                )),
                error: "Main class Id is not allow".to_string(),
            });
        }
    }

    match state
        .db
        .subjects
        .create_new(item.into_inner(), "subject")
        .await
    {
        Err(e) => HttpResponse::BadRequest().json(RequestErrorModel {
            error: e.to_string(),
            message: Some("Something went wrong to create subject , try again late".to_string()),
        }),
        Ok(doc) => {
            HttpResponse::Created().json(convert_fields_to_string(to_document(&doc).unwrap()))
        }
    }
}
