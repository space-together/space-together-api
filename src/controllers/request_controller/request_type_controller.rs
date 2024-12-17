use std::sync::Arc;

use crate::{
    error::request_error::request_error_error::RequestRequest,
    models::request_model::request_type_model::{
        RequestTypeModel, RequestTypeModelGet, RequestTypeModelNew,
    },
    AppState,
};

pub async fn controllers_request_type_create(
    state: Arc<AppState>,
    role: RequestTypeModelNew,
) -> RequestRequest<RequestTypeModelGet> {
    match state.db.request_type.create(role.clone()).await {
        Err(e) => Err(e),
        Ok(_) => match state.db.request_type.get_by_role(role.role).await {
            Ok(res) => Ok(RequestTypeModel::format(res)),
            Err(e) => Err(e),
        },
    }
}

pub async fn controllers_request_type_get_all(
    state: Arc<AppState>,
) -> RequestRequest<Vec<RequestTypeModelGet>> {
    match state.db.request_type.get_all().await {
        Err(e) => Err(e),
        Ok(res) => Ok(res),
    }
}
