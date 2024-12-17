use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    results::InsertOneResult,
    Collection,
};

use crate::{
    error::request_error::request_error_error::{RequestError, RequestRequest},
    models::request_model::request_model_model::{RequestModel, RequestModelGet, RequestModelNew},
};

#[derive(Debug)]
pub struct RequestDb {
    pub request: Collection<RequestModel>,
}

impl RequestDb {
    pub async fn create(&self, request: RequestModelNew) -> RequestRequest<InsertOneResult> {
        match self.request.insert_one(RequestModel::new(request)).await {
            Ok(id) => Ok(id),
            Err(err) => Err(RequestError::CanDoAction {
                action: "create".to_string(),
                error: err.to_string(),
            }),
        }
    }
    pub async fn get_by_id(&self, id: ObjectId) -> RequestRequest<RequestModelGet> {
        match self.request.find_one(doc! { "_id": &id}).await {
            Ok(Some(res)) => Ok(RequestModel::format(res)),
            Ok(None) => Err(RequestError::CanNotGetRequestByField {
                field: "_id".to_string(),
                value: id.to_string(),
            }),
            Err(err) => Err(RequestError::CanDoAction {
                action: "get".to_string(),
                error: err.to_string(),
            }),
        }
    }

    pub async fn get_all(&self) -> RequestRequest<Vec<RequestModelGet>> {
        let cursor = self.request.find(doc! {}).sort(doc! {"co" : -1}).await;

        let mut requests = Vec::new();

        match cursor {
            Ok(mut res) => {
                while let Some(result) = res.next().await {
                    match result {
                        Ok(doc) => requests.push(RequestModel::format(doc)),
                        Err(e) => {
                            return Err(RequestError::CanDoAction {
                                action: "get all request".to_string(),
                                error: e.to_string(),
                            })
                        }
                    }
                }
                Ok(requests)
            }
            Err(e) => Err(RequestError::CanDoAction {
                action: "get all request 😉".to_string(),
                error: e.to_string(),
            }),
        }
    }

    pub async fn delete(&self, id: ObjectId) -> RequestRequest<RequestModelGet> {
        match self.request.find_one_and_delete(doc! {"_id" : id}).await {
            Ok(Some(res)) => Ok(RequestModel::format(res)),
            Ok(None) => Err(RequestError::CanNotGetRequestByField {
                field: "_id".to_string(),
                value: id.to_string(),
            }),
            Err(err) => Err(RequestError::CanDoAction {
                action: "delete".to_string(),
                error: err.to_string(),
            }),
        }
    }
}
