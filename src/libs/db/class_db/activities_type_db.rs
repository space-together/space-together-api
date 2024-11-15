use std::str::FromStr;

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
        ActivitiesTypeModel, ActivitiesTypeModelGet, ActivitiesTypeModelNew,
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

        let one_index = self.activities_type.create_index(index).await;
        if one_index.is_err() {
            return Err(ActivitiesTypeErr::ActivitiesTypeIsReadyExit);
        };

        let new = ActivitiesTypeModel::new(activity_type);
        let create = self.activities_type.insert_one(new).await;

        match create {
            Ok(res) => Ok(res),
            Err(err) => Err(ActivitiesTypeErr::CanNotCreateActivitiesType {
                err: err.to_string(),
            }),
        }
    }
    pub async fn get_activities_type_by_id(
        &self,
        id: String,
    ) -> ActivitiesTypeResult<ActivitiesTypeModel> {
        let obj_id = ObjectId::from_str(&id).map_err(|_| ActivitiesTypeErr::InvalidId);

        match obj_id {
            Ok(i) => {
                let get = self.activities_type.find_one(doc! {"_id" : i}).await;
                match get {
                    Ok(Some(data)) => Ok(data),
                    Ok(None) => Err(ActivitiesTypeErr::ActivitiesTypeNotFound),
                    Err(er) => Err(ActivitiesTypeErr::CanNotFindActivitiesType {
                        err: er.to_string(),
                    }),
                }
            }
            Err(err) => Err(err),
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
}
