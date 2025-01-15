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
    pub sections: Option<Vec<ObjectId>>,
    pub code: Option<String>,
    pub subjects: Option<Vec<ObjectId>>,
    pub rooms: Option<Vec<ObjectId>>,
    pub class_type_id: Option<ObjectId>,
    pub is_public: Option<bool>,
    pub image: Option<ObjectId>,
    pub create_on: DateTime,         // create on
    pub update_on: Option<DateTime>, // update on
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModelNew {
    pub name: String,
    pub class_teacher_id: String,
    pub code: Option<String>,
    pub sections: Option<Vec<String>>,
    pub subjects: Option<Vec<String>>,
    pub rooms: Option<Vec<String>>,
    pub image: Option<String>,
    pub teachers: Option<Vec<String>>,
    pub students: Option<Vec<String>>,
    pub class_type_id: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassModelPut {
    pub name: Option<String>,
    pub class_teacher_id: Option<String>, // teacher id
    pub students: Option<Vec<String>>,    // student id
    pub teachers: Option<Vec<String>>,    // teachers id
    pub sections: Option<Vec<String>>,
    pub rooms: Option<Vec<String>>,
    pub image: Option<String>,
    pub subjects: Option<Vec<String>>,
    pub code: Option<String>,
    pub class_type_id: Option<String>,
    pub is_public: Option<bool>,
}

impl ClassModel {
    pub fn new(class: ClassModelNew) -> Self {
        ClassModel {
            id: None,
            name: class.name,
            is_public: class.is_public,
            image: class.image.map(|r| ObjectId::from_str(&r).unwrap()),
            class_type_id: class.class_type_id.map(|r| ObjectId::from_str(&r).unwrap()),
            class_teacher_id: ObjectId::from_str(&class.class_teacher_id).unwrap(),
            code: class.code,
            subjects: class.subjects.map(|ids| {
                ids.iter()
                    .map(|id| ObjectId::from_str(id).unwrap())
                    .collect()
            }),
            sections: class.sections.map(|ids| {
                ids.iter()
                    .map(|id| ObjectId::from_str(id).unwrap())
                    .collect()
            }),
            rooms: class.rooms.map(|ids| {
                ids.iter()
                    .map(|id| ObjectId::from_str(id).unwrap())
                    .collect()
            }),
            teachers: class.teachers.map(|ids| {
                ids.iter()
                    .map(|id| ObjectId::from_str(id).unwrap())
                    .collect()
            }),
            students: class.students.map(|ids| {
                ids.iter()
                    .map(|id| ObjectId::from_str(id).unwrap())
                    .collect()
            }),
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

        insert_if_some("is_public", class.is_public.map(bson::Bson::Boolean));
        insert_if_some("name", class.name.map(bson::Bson::String));
        insert_if_some(
            "class_teacher_id",
            class
                .class_teacher_id
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );
        insert_if_some(
            "image",
            class
                .image
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );
        insert_if_some(
            "class_type_id",
            class
                .class_type_id
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
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
            "rooms",
            class.rooms.map(|students| {
                bson::Bson::Array(
                    students
                        .into_iter()
                        .filter_map(|id| ObjectId::from_str(&id).ok().map(bson::Bson::ObjectId))
                        .collect(),
                )
            }),
        );
        insert_if_some(
            "subjects",
            class.subjects.map(|students| {
                bson::Bson::Array(
                    students
                        .into_iter()
                        .filter_map(|id| ObjectId::from_str(&id).ok().map(bson::Bson::ObjectId))
                        .collect(),
                )
            }),
        );
        insert_if_some(
            "sections",
            class.sections.map(|students| {
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
    pub image: Option<String>,
    pub is_public: Option<bool>,
    pub sections: Option<Vec<String>>,
    pub subjects: Option<Vec<String>>,
    pub rooms: Option<String>,
    pub class_type_id: Option<String>,
    pub code: Option<String>,
    pub create_on: String,
    pub update_on: Option<String>,
}

impl ClassModelGet {
    pub fn format(class: ClassModel) -> ClassModelGet {
        ClassModelGet {
            id: class.id.map_or("".to_string(), |id| id.to_string()),
            name: class.name,
            is_public: class.is_public,
            image: class.image.map(|i| i.to_string()),
            class_type_id: class.class_type_id.map(|i| i.to_string()),
            code: class.code,
            subjects: class
                .subjects
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
            rooms: class
                .rooms
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
            sections: class
                .sections
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
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
