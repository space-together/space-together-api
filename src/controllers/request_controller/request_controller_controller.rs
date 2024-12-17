use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::request_error::request_error_error::RequestRequest,
    libs::functions::object_id::change_insertoneresult_into_object_id,
    models::request_model::request_model_model::{RequestModelGet, RequestModelNew},
    AppState,
};

pub async fn controller_request_create(
    state: Arc<AppState>,
    request: RequestModelNew,
) -> RequestRequest<RequestModelGet> {
    match state.db.request.create(request).await {
        Err(e) => Err(e),
        Ok(id) => match state
            .db
            .request
            .get_by_id(change_insertoneresult_into_object_id(id))
            .await
        {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        },
    }
}

pub async fn controller_request_get_all(
    state: Arc<AppState>,
) -> RequestRequest<Vec<RequestModelGet>> {
    state.db.request.get_all().await
}

pub async fn controller_request_delete(
    state: Arc<AppState>,
    id: ObjectId,
) -> RequestRequest<RequestModelGet> {
    state.db.request.delete(id).await
}
pub async fn controller_request_get_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> RequestRequest<RequestModelGet> {
    state.db.request.get_by_id(id).await
}
