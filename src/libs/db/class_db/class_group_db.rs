use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    results::InsertOneResult,
    Collection,
};

use crate::{
    error::class_error::class_group_err::{ClassGroupErr, ClassGroupResult},
    models::class_model::class_group_model::class_group_model_model::{
        ClassGroupModel, ClassGroupModelGet, ClassGroupModelNew,
    },
};

#[derive(Debug)]
pub struct ClassGroupDb {
    pub class_group: Collection<ClassGroupModel>,
}

impl ClassGroupDb {
    pub async fn class_group_create(
        &self,
        group: ClassGroupModelNew,
    ) -> ClassGroupResult<InsertOneResult> {
        let new = ClassGroupModel::new(group);

        match new {
            Ok(res) => {
                let create = self.class_group.insert_one(res).await;
                match create {
                    Ok(result) => Ok(result),
                    Err(err) => Err(ClassGroupErr::CanNotCreateClassGroup {
                        err: err.to_string(),
                    }),
                }
            }
            Err(err) => Err(err),
        }
    }

    async fn get_class_by_filed(
        &self,
        filed: &str,
        value: ObjectId,
    ) -> ClassGroupResult<ClassGroupModel> {
        match self.class_group.find_one(doc! {filed : value}).await {
            Ok(Some(group)) => Ok(group),
            Ok(None) => Err(ClassGroupErr::ClassGroupNotFoundById),
            Err(err) => Err(ClassGroupErr::CanNotFindClassGroup {
                err: err.to_string(),
            }),
        }
    }

    pub async fn get_class_group_by_id(&self, id: ObjectId) -> ClassGroupResult<ClassGroupModel> {
        self.get_class_by_filed("_id", id).await
    }

    pub async fn get_all_class_group(&self) -> ClassGroupResult<Vec<ClassGroupModelGet>> {
        let cursor = self.class_group.find(doc! {}).await.map_err(|err| {
            ClassGroupErr::CanNotGetAllClassGroups {
                err: err.to_string(),
            }
        });
        let mut groups: Vec<ClassGroupModelGet> = Vec::new();

        match cursor {
            Ok(mut res) => {
                while let Some(result) = res.next().await {
                    match result {
                        Ok(group) => groups.push(ClassGroupModel::format(group)),
                        Err(err) => {
                            return Err(ClassGroupErr::CanNotGetAllClassGroups {
                                err: err.to_string(),
                            })
                        }
                    }
                }
                Ok(groups)
            }
            Err(err) => Err(err),
        }
    }

    async fn find_many_by_field(
        &self,
        field: &str,
        value: ObjectId,
    ) -> ClassGroupResult<Vec<ClassGroupModel>> {
        let query = if field == "st" {
            doc! { "st": { "$in": [value] } }
        } else {
            doc! { field: value }
        };

        let mut cursor = self.class_group.find(query).await.map_err(|err| {
            ClassGroupErr::CanNotGetAllClassGroupBy {
                err: err.to_string(),
                field: field.to_string(),
            }
        })?;

        let mut groups = Vec::new();
        while let Some(data) = cursor.next().await {
            match data {
                Ok(doc) => groups.push(doc),
                Err(err) => {
                    return Err(ClassGroupErr::CanNotGetAllClassGroupBy {
                        err: err.to_string(),
                        field: field.to_string(),
                    });
                }
            }
        }

        Ok(groups)
    }

    pub async fn get_class_group_by_class(
        &self,
        class: ObjectId,
    ) -> ClassGroupResult<Vec<ClassGroupModel>> {
        self.find_many_by_field("cl_id", class).await
    }
}
