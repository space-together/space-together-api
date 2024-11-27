use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    error::class_error::activities_type_error::{ActivitiesTypeErr, ActivitiesTypeResult},
    models::class_model::activities_type_model::{
        ActivitiesTypeModel, ActivitiesTypeModelGet, ActivitiesTypeModelNew, ActivitiesTypeModelPut,
    },
};

#[derive(Debug)]
pub struct ActivitiesTypeDb {
    pub activities_type: Collection<ActivitiesTypeModel>,
}

impl ActivitiesTypeDb {
    pub async fn create_activity_type(
        &self,
        activity_type: ActivitiesTypeModelNew,
    ) -> ActivitiesTypeResult<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
                "ty" : 1,
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        if self.activities_type.create_index(index).await.is_err() {
            return Err(ActivitiesTypeErr::ActivitiesTypeIsReadyExit {
                name: activity_type.ty.clone(),
            });
        };

        match self
            .activities_type
            .insert_one(ActivitiesTypeModel::new(activity_type))
            .await
        {
            Ok(res) => Ok(res),
            Err(err) => Err(ActivitiesTypeErr::CanNotCreateActivitiesType {
                err: err.to_string(),
            }),
        }
    }
    pub async fn get_activities_type_by_id(
        &self,
        id: ObjectId,
    ) -> ActivitiesTypeResult<ActivitiesTypeModel> {
        match self.activities_type.find_one(doc! {"_id" : id}).await {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(ActivitiesTypeErr::ActivitiesTypeNotFound),
            Err(er) => Err(ActivitiesTypeErr::CanNotFindActivitiesType {
                err: er.to_string(),
            }),
        }
    }

    pub async fn get_all_activities_type(
        &self,
    ) -> ActivitiesTypeResult<Vec<ActivitiesTypeModelGet>> {
        let cursor = self.activities_type.find(doc! {}).await.map_err(|err| {
            ActivitiesTypeErr::CanNotGetAllActivitiesTypes {
                err: err.to_string(),
            }
        });
        let mut activities_types: Vec<ActivitiesTypeModelGet> = Vec::new();

        match cursor {
            Ok(mut res) => {
                while let Some(result) = res.next().await {
                    match result {
                        Ok(doc) => activities_types.push(ActivitiesTypeModel::format(doc)),
                        Err(err) => {
                            return Err(ActivitiesTypeErr::CanNotGetAllActivitiesTypes {
                                err: err.to_string(),
                            })
                        }
                    }
                }
                Ok(activities_types)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_activities_type_by_ty(
        &self,
        ty: String,
    ) -> ActivitiesTypeResult<ActivitiesTypeModel> {
        let get = self.activities_type.find_one(doc! {"ty" : &ty}).await;
        match get {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(ActivitiesTypeErr::ActivitiesTypeNotFound),
            Err(er) => Err(ActivitiesTypeErr::CanNotFindActivitiesType {
                err: er.to_string(),
            }),
        }
    }

    pub async fn delete_activities_type_by_id(
        &self,
        id: ObjectId,
    ) -> ActivitiesTypeResult<ActivitiesTypeModel> {
        match self
            .activities_type
            .find_one_and_delete(doc! {"_id" : id})
            .await
        {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(ActivitiesTypeErr::ActivitiesTypeNotFound),
            Err(er) => Err(ActivitiesTypeErr::CanNotDoAction {
                err: er.to_string(),
                action: "delete".to_string(),
            }),
        }
    }

    pub async fn update_activities_type_by_id(
        &self,
        id: ObjectId,
        activity_type: ActivitiesTypeModelPut,
    ) -> ActivitiesTypeResult<ActivitiesTypeModel> {
        match self
            .activities_type
            .find_one_and_update(
                doc! {"_id" : id},
                doc! {"$set" : ActivitiesTypeModel::put(activity_type)},
            )
            .await
        {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(ActivitiesTypeErr::ActivitiesTypeNotFound),
            Err(er) => Err(ActivitiesTypeErr::CanNotDoAction {
                err: er.to_string(),
                action: "update".to_string(),
            }),
        }
    }
}
