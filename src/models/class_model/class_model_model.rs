use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,                    // name
    pub class_teacher_id: ObjectId,      // teacher id
    pub students: Option<Vec<ObjectId>>, // Student
    pub teachers: Option<Vec<ObjectId>>, //teachers
    pub section: Option<ObjectId>,
    pub code: Option<String>,
    pub subjects: Option<Vec<ObjectId>>,
    pub room: Option<ObjectId>,
    pub image: Option<ObjectId>,
    pub create_on: DateTime,         // create on
    pub update_on: Option<DateTime>, // update on
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModelNew {
    pub name: String,
    pub class_teacher_id: String,
    pub code: Option<String>,
    pub section: Option<String>,
    pub subjects: Option<Vec<String>>,
    pub room: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModelPut {
    pub name: Option<String>,
    pub class_teacher_id: Option<String>, // teacher id
    pub students: Option<Vec<String>>,    // student id
    pub teachers: Option<Vec<String>>,    // teachers id
}

impl ClassModel {
    pub fn new(class: ClassModelNew) -> Self {
        ClassModel {
            id: None,
            name: class.name,
            image: class.image.map(|r| ObjectId::from_str(&r).unwrap()),
            class_teacher_id: ObjectId::from_str(&class.class_teacher_id).unwrap(),
            room: class.room.map(|r| ObjectId::from_str(&r).unwrap()),
            code: class.code,
            subjects: class.subjects.map(|ids| {
                ids.iter()
                    .filter_map(|id| Some(ObjectId::from_str(id).unwrap()))
                    .collect()
            }),
            section: class.section.map(|s| ObjectId::from_str(&s).unwrap()),
            teachers: Some(Vec::new()),
            students: Some(Vec::new()),
            create_on: DateTime::now(),
            update_on: None,
        }
    }

    pub fn put(class: ClassModelPut) -> Document {
        let mut doc = Document::new();

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
            }
        };

        // Handle updates for `name` (name) and `class_teacher_id` (class teacher)
        insert_if_some("name", class.name.map(bson::Bson::String));
        insert_if_some(
            "class_teacher_id",
            class.class_teacher_id.map(|class_teacher_id| {
                bson::Bson::ObjectId(ObjectId::from_str(&class_teacher_id).unwrap())
            }),
        );

        insert_if_some(
            "students",
            class.students.map(|students| {
                bson::Bson::Array(
                    students
                        .into_iter()
                        .filter_map(|id| ObjectId::from_str(&id).ok().map(bson::Bson::ObjectId))
                        .collect(),
                )
            }),
        );

        insert_if_some(
            "teachers",
            class.teachers.map(|teachers| {
                bson::Bson::Array(
                    teachers
                        .into_iter()
                        .filter_map(|id| ObjectId::from_str(&id).ok().map(bson::Bson::ObjectId))
                        .collect(),
                )
            }),
        );

        doc
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModelGet {
    pub id: String,
    pub name: String,
    pub class_teacher_id: String,
    pub students: Option<Vec<String>>,
    pub teachers: Option<Vec<String>>,
    pub create_on: String,
    pub update_on: Option<String>,
}

impl ClassModelGet {
    pub fn format(class: ClassModel) -> ClassModelGet {
        ClassModelGet {
            id: class.id.map_or("".to_string(), |id| id.to_string()),
            name: class.name,
            class_teacher_id: class.class_teacher_id.to_string(),
            students: class
                .students
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
            teachers: class
                .teachers
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
            create_on: class
                .create_on
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
            update_on: Some(class.update_on.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }
}
