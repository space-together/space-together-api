use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    error::request_error::request_error_error::{RequestError, RequestRequest},
    models::request_model::request_type_model::{
        RequestTypeModel, RequestTypeModelGet, RequestTypeModelNew, RequestTypeModelPut,
    },
};

#[derive(Debug)]
pub struct RequestTypeDb {
    pub request: Collection<RequestTypeModel>,
}

impl RequestTypeDb {
    pub async fn find_one_by_field(
        &self,
        field: &str,
        value: String,
    ) -> RequestRequest<RequestTypeModel> {
        // if field == "_id" {
        //     field = ObjectId::from_str(field);
        // }

        match self.request.find_one(doc! { field: &value}).await {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(RequestError::CanNotGetRequestByField {
                field: field.to_string(),
                value,
            }),
            Err(err) => Err(RequestError::CanDoAction {
                action: "get".to_string(),
                error: err.to_string(),
            }),
        }
    }

    // pub async fn get_by_id(&self, id: ObjectId) -> RequestRequest<RequestTypeModel> {
    //     todo!()
    // }

    pub async fn get_by_role(&self, role: String) -> RequestRequest<RequestTypeModel> {
        self.find_one_by_field("role", role).await
    }

    pub async fn create(&self, role: RequestTypeModelNew) -> RequestRequest<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
            "role" : 1,
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        if let Err(e) = self.request.create_index(index).await {
            return Err(RequestError::CanDoAction {
                action: "Index".to_string(),
                error: e.to_string(),
            });
        }

        if self.get_by_role(role.role.clone()).await.is_ok() {
            return Err(RequestError::CanDoAction {
                action: "create request type".to_string(),
                error: "it ready exit".to_string(),
            });
        }

        match self.request.insert_one(RequestTypeModel::new(role)).await {
            Ok(id) => Ok(id),
            Err(err) => Err(RequestError::CanDoAction {
                action: "create".to_string(),
                error: err.to_string(),
            }),
        }
    }

    pub async fn put(
        &self,
        role: RequestTypeModelPut,
        id: ObjectId,
    ) -> RequestRequest<RequestTypeModel> {
        match self
            .request
            .find_one_and_update(
                doc! {"_id" : id},
                doc! {"$set" : RequestTypeModel::put(role)},
            )
            .await
        {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(RequestError::CanNotGetRequestByField {
                field: "_id".to_string(),
                value: id.to_string(),
            }),
            Err(err) => Err(RequestError::CanDoAction {
                action: "update".to_string(),
                error: err.to_string(),
            }),
        }
    }

    pub async fn delete(&self, id: ObjectId) -> RequestRequest<RequestTypeModel> {
        match self.request.find_one_and_delete(doc! {"_id" : id}).await {
            Ok(Some(res)) => Ok(res),
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

    pub async fn get_all(&self) -> RequestRequest<Vec<RequestTypeModelGet>> {
        let mut roles: Vec<RequestTypeModelGet> = Vec::new();

        match self.request.find(doc! {}).await {
            Ok(mut res) => {
                while let Some(result) = res.next().await {
                    match result {
                        Ok(doc) => roles.push(RequestTypeModel::format(doc)),
                        Err(e) => {
                            return Err(RequestError::CanDoAction {
                                action: "get all request".to_string(),
                                error: e.to_string(),
                            })
                        }
                    }
                }
                Ok(roles)
            }
            Err(e) => Err(RequestError::CanDoAction {
                action: "get all request".to_string(),
                error: e.to_string(),
            }),
        }
    }
}
