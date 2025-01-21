// use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct FileModel {
//     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//     pub id: Option<ObjectId>,
//     pub name: String,
//     pub description: Option<String>,
//     pub created_on: DateTime,
//     pub updated_on: Option<DateTime>,
// }

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct FileModelGet {
//     pub id: String,
//     pub name: String,
//     pub description: Option<String>,
//     pub created_on: String,
//     pub updated_on: Option<String>,
// }

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct FileModelNew {
//     pub name: String,
//     pub description: Option<String>,
// }

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct FileModelPut {
//     pub name: Option<String>,
//     pub description: Option<String>,
// }

// impl FileModel {
//     pub fn new(class_type: FileModelNew) -> Self {
//         FileModel {
//             id: None,
//             name: class_type.name,
//             description: class_type.description,
//             created_on: DateTime::now(),
//             updated_on: None,
//         }
//     }

//     pub fn format(class_type: Self) -> FileModelGet {
//         FileModelGet {
//             id: class_type.id.map_or("".to_string(), |id| id.to_string()),
//             name: class_type.name,
//             description: class_type.description,
//             created_on: class_type
//                 .created_on
//                 .try_to_rfc3339_string()
//                 .unwrap_or("".to_string()),
//             updated_on: Some(class_type.updated_on.map_or("".to_string(), |date| {
//                 date.try_to_rfc3339_string().unwrap_or("".to_string())
//             })),
//         }
//     }

//     pub fn put(class_type: FileModelPut) -> Document {
//         let mut doc = Document::new();
//         let mut is_update = false;

//         let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
//             if let Some(v) = value {
//                 doc.insert(key, v);
//                 is_update = true;
//             }
//         };

//         insert_if_some("name", class_type.name.map(bson::Bson::String));
//         insert_if_some(
//             "description",
//             class_type.description.map(bson::Bson::String),
//         );

//         if is_update {
//             doc.insert("updated_on", bson::Bson::DateTime(DateTime::now()));
//         }

//         doc
//     }
// }
