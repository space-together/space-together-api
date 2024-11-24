use std::str::FromStr;

use futures::stream::StreamExt;

use mongodb::{
    bson::{doc, oid::ObjectId},
    results::InsertOneResult,
    Collection,
};

use crate::{
    error::class_error::class_error_error::{ClassError, ClassResult},
    models::class_model::class_model_model::{
        ClassModel, ClassModelGet, ClassModelNew, ClassModelPut,
    },
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

    pub async fn get_class_by_id(&self, id: ObjectId) -> ClassResult<ClassModel> {
        match self.class.find_one(doc! {"_id" : id}).await {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Err(ClassError::ClassNotFoundById),
            Err(err) => Err(ClassError::CanNotGetClass {
                err: err.to_string(),
            }),
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

    pub async fn update_class(
        &self,
        class: Option<ClassModelPut>,
        id: ObjectId,
        add_students: Option<Vec<String>>,
        remove_students: Option<Vec<String>>,
    ) -> ClassResult<ClassModel> {
        let mut update_doc = doc! {
            "$currentDate": { "uo": true },
        };

        if let Some(class_data) = class {
            update_doc.insert("$set", ClassModel::put(class_data));
        }

        if let Some(add) = add_students {
            let student_obj_ids: Vec<ObjectId> = add
                .into_iter()
                .filter_map(|id| ObjectId::from_str(&id).ok())
                .collect();

            if !student_obj_ids.is_empty() {
                update_doc.insert(
                    "$addToSet",
                    doc! {
                        "st": { "$each": student_obj_ids }
                    },
                );
            }
        }

        if let Some(remove) = remove_students {
            let student_obj_ids: Vec<ObjectId> = remove
                .into_iter()
                .filter_map(|id| ObjectId::from_str(&id).ok())
                .collect();

            if !student_obj_ids.is_empty() {
                update_doc.insert(
                    "$pullAll",
                    doc! {
                        "st": student_obj_ids
                    },
                );
            }
        }

        match self
            .class
            .find_one_and_update(doc! { "_id": id }, update_doc)
            .await
        {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Err(ClassError::ClassNotFoundById),
            Err(err) => Err(ClassError::CanNotUpdateClass {
                err: err.to_string(),
            }),
        }
    }
}
