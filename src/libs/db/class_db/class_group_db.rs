use std::str::FromStr;

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

    pub async fn get_class_group_by_id(&self, id: String) -> ClassGroupResult<ClassGroupModel> {
        let doc = ObjectId::from_str(&id);
        match doc {
            Ok(res) => {
                let get = self.class_group.find_one(doc! {"_id" : res}).await;
                match get {
                    Ok(Some(result)) => Ok(result),
                    Ok(None) => Err(ClassGroupErr::ClassGroupNotFoundById),
                    Err(err) => Err(ClassGroupErr::CanNotFindClassGroup {
                        err: err.to_string(),
                    }),
                }
            }
            Err(_) => Err(ClassGroupErr::InvalidId),
        }
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
}
