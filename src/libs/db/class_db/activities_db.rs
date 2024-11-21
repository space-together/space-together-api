use crate::{
    error::class_error::activities_error::{ActivitiesErr, ActivitiesResult},
    models::class_model::activity_model::{ActivityModel, ActivityModelNew},
};
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    results::InsertOneResult,
    Collection,
};

#[derive(Debug)]
pub struct ActivityDb {
    pub activity: Collection<ActivityModel>,
}

impl ActivityDb {
    pub async fn create_activity(
        &self,
        activity: ActivityModelNew,
    ) -> ActivitiesResult<InsertOneResult> {
        match ActivityModel::new(activity) {
            Ok(doc) => self.activity.insert_one(doc).await.map_err(|err| {
                ActivitiesErr::CanCreateActivity {
                    error: err.to_string(),
                }
            }),
            Err(err) => Err(err),
        }
    }

    async fn find_one_by_field(
        &self,
        field: &str,
        value: ObjectId,
    ) -> ActivitiesResult<Option<ActivityModel>> {
        self.activity
            .find_one(doc! { field: value })
            .await
            .map_err(|err| ActivitiesErr::CanNotFindActivity {
                error: err.to_string(),
            })
    }

    async fn find_many_by_field(
        &self,
        field: &str,
        value: ObjectId,
    ) -> ActivitiesResult<Vec<ActivityModel>> {
        let mut cursor = self
            .activity
            .find(doc! { field: value })
            .await
            .map_err(|err| ActivitiesErr::CanGetAllActivity {
                error: err.to_string(),
                field: field.to_string().clone(),
            })?;

        let mut activities = Vec::new();
        while let Some(data) = cursor.next().await {
            match data {
                Ok(doc) => activities.push(doc),
                Err(err) => {
                    return Err(ActivitiesErr::CanGetAllActivity {
                        error: err.to_string(),
                        field: field.to_string(),
                    });
                }
            }
        }
        Ok(activities)
    }

    pub async fn get_activity_by_id(&self, id: ObjectId) -> ActivitiesResult<ActivityModel> {
        match self.find_one_by_field("_id", id).await? {
            Some(activity) => Ok(activity),
            None => Err(ActivitiesErr::ActivityNotFound),
        }
    }

    pub async fn get_activity_by_class(
        &self,
        id: ObjectId,
    ) -> ActivitiesResult<Vec<ActivityModel>> {
        self.find_many_by_field("cl", id).await
    }

    pub async fn get_activity_by_group(
        &self,
        id: ObjectId,
    ) -> ActivitiesResult<Vec<ActivityModel>> {
        self.find_many_by_field("gr", id).await
    }
}
