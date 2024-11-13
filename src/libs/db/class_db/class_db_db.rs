use futures::stream::StreamExt;
use std::str::FromStr;

use mongodb::{
    bson::{doc, oid::ObjectId},
    results::InsertOneResult,
    Collection,
};

use crate::{
    error::class_error::class_error_error::{ClassError, ClassResult},
    models::class_model::class_model_model::{ClassModel, ClassModelGet, ClassModelNew},
};

#[derive(Debug)]
pub struct ClassDb {
    pub class: Collection<ClassModel>,
}

impl ClassDb {
    pub async fn create_class(&self, class: ClassModelNew) -> ClassResult<InsertOneResult> {
        let new = ClassModel::new(class);

        match new {
            Ok(res) => {
                let create = self.class.insert_one(res).await;
                match create {
                    Ok(result) => Ok(result),
                    Err(err) => Err(ClassError::CanCreateClass {
                        err: err.to_string(),
                    }),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_class_by_id(&self, id: String) -> ClassResult<ClassModel> {
        let obj_id = ObjectId::from_str(&id).map_err(|_| ClassError::InvalidId);
        match obj_id {
            Ok(res) => {
                let get = self.class.find_one(doc! {"_id" : res}).await;
                match get {
                    Ok(Some(result)) => Ok(result),
                    Ok(None) => Err(ClassError::ClassNotFoundById),
                    Err(err) => Err(ClassError::CanNotGetClass {
                        err: err.to_string(),
                    }),
                }
            }
            Err(_) => Err(ClassError::InvalidId),
        }
    }

    pub async fn get_all_classes(&self) -> ClassResult<Vec<ClassModelGet>> {
        let cursor = self
            .class
            .find(doc! {})
            .await
            .map_err(|err| ClassError::CanNotGetAllClass {
                err: err.to_string(),
            });
        let mut classes: Vec<ClassModelGet> = Vec::new();

        match cursor {
            Ok(mut res) => {
                while let Some(result) = res.next().await {
                    match result {
                        Ok(doc) => classes.push(ClassModelGet::format(doc)),
                        Err(err) => {
                            return Err(ClassError::CanNotGetAllClass {
                                err: err.to_string(),
                            })
                        }
                    }
                }
                Ok(classes)
            }
            Err(err) => Err(err),
        }
    }
}
