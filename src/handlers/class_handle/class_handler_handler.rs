use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::Deserialize;

use crate::{
    controllers::class_controller::class_controller_controller::{
        controller_class_delete_by_id, controller_class_gets_by_student,
        controller_class_gets_by_teacher, controller_class_update, controller_create_class,
        controller_get_all_classes, controller_get_class_by_id,
    },
    libs::functions::object_id::change_string_into_object_id,
    models::{
        class_model::class_model_model::{ClassModelNew, ClassModelPut},
        request_error_model::ReqErrModel,
    },
    AppState,
};

pub async fn handle_create_class(
    state: Data<AppState>,
    class: Json<ClassModelNew>,
) -> impl Responder {
    let create = controller_create_class(state.into_inner(), class.into_inner()).await;
    match create {
        Ok(res) => HttpResponse::Created().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_get_class_by_id(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => match controller_get_class_by_id(state.into_inner(), _id).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handle_class_delete_by_id(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => match controller_class_delete_by_id(state.into_inner(), _id).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handle_class_gets_by_teacher(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => match controller_class_gets_by_teacher(state.into_inner(), _id).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handle_class_gets_by_student(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(err) => HttpResponse::BadRequest().json(err),
        Ok(_id) => match controller_class_gets_by_student(state.into_inner(), _id).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
                message: err.to_string(),
            }),
        },
    }
}

pub async fn handler_get_all_classes(state: Data<AppState>) -> impl Responder {
    let all = controller_get_all_classes(state.into_inner()).await;
    match all {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct AddRemoveStudentsModel {
    pub students: Vec<String>,
}

pub async fn handle_class_add_students(
    state: Data<AppState>,
    id: Path<String>,
    request: Json<AddRemoveStudentsModel>,
) -> impl Responder {
    let class_id = match change_string_into_object_id(id.into_inner()) {
        Err(err) => return HttpResponse::BadRequest().json(err),
        Ok(id) => id,
    };

    let students = request.into_inner().students;

    if students.is_empty() {
        return HttpResponse::BadRequest().json(ReqErrModel {
            message: "students is empty".to_string(),
        });
    }

    match controller_class_update(state.into_inner(), class_id, None, Some(students), None).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_class_remove_students(
    state: Data<AppState>,
    id: Path<String>,
    request: Json<AddRemoveStudentsModel>,
) -> impl Responder {
    let class_id = match change_string_into_object_id(id.into_inner()) {
        Err(err) => return HttpResponse::BadRequest().json(err),
        Ok(id) => id,
    };

    let student = request.into_inner().students;

    if student.is_empty() {
        return HttpResponse::BadRequest().json(ReqErrModel {
            message: "student is empty".to_string(),
        });
    }

    match controller_class_update(state.into_inner(), class_id, None, None, Some(student)).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}

pub async fn handle_class_update_by_id(
    state: Data<AppState>,
    id: Path<String>,
    request: Json<ClassModelPut>,
) -> impl Responder {
    let class_id = match change_string_into_object_id(id.into_inner()) {
        Err(err) => return HttpResponse::BadRequest().json(err),
        Ok(id) => id,
    };

    match controller_class_update(
        state.into_inner(),
        class_id,
        Some(request.into_inner()),
        None,
        None,
    )
    .await
    {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(err) => HttpResponse::BadRequest().json(ReqErrModel {
            message: err.to_string(),
        }),
    }
}
