use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

use crate::error::class_error::activities_error::{ActivitiesErr, ActivitiesResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivityModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub ty: ObjectId,               // Activity type
    pub create_by: ObjectId,        // create by
    pub group: Option<ObjectId>,    // group
    pub class_id: Option<ObjectId>, // class id
    pub act: String,                // activity description
    pub dl: DateTime,               // died line
    pub co: DateTime,               // created at
    pub uo: Option<DateTime>,       // Update on
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivityModelGet {
    pub id: String,
    pub ty: String,               // Activity type
    pub create_by: String,        // create by
    pub act: String,              // activity description
    pub dl: String,               // died line
    pub group: Option<String>,    // groupoup
    pub class_id: Option<String>, // class id
    pub co: String,               // created at
    pub uo: Option<String>,       // Update on
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivityModelPut {
    pub ty: Option<String>,
    pub act: Option<String>,
    pub dl: Option<String>,
    pub group: Option<String>,
    pub class_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivityModelNew {
    pub ty: String,
    pub class_id: Option<String>,
    pub group: Option<String>,
    pub create_by: String,
    pub act: String,
    pub dl: String,
}

impl ActivityModel {
    pub fn new(activity: ActivityModelNew) -> ActivitiesResult<Self> {
        let ty = ObjectId::from_str(&activity.ty).map_err(|_| ActivitiesErr::Invalid)?;
        let create_by =
            ObjectId::from_str(&activity.create_by).map_err(|_| ActivitiesErr::Invalid)?;
        let class_id = match &activity.class_id {
            Some(class_id) => {
                Some(ObjectId::from_str(class_id).map_err(|_| ActivitiesErr::Invalid)?)
            }
            None => None,
        };
        let group = match &activity.group {
            Some(group) => Some(ObjectId::from_str(group).map_err(|_| ActivitiesErr::Invalid)?),
            None => None,
        };
        let dl = DateTime::parse_rfc3339_str(&activity.dl).unwrap();

        Ok(ActivityModel {
            id: None,
            create_by,
            ty,
            class_id,
            group,
            act: activity.act,
            dl,
            co: DateTime::now(),
            uo: None,
        })
    }

    pub fn format(activity: ActivityModel) -> ActivityModelGet {
        ActivityModelGet {
            id: activity.id.map_or("".to_string(), |id| id.to_string()),
            act: activity.act,
            class_id: Some(
                activity
                    .class_id
                    .map_or("".to_string(), |id| id.to_string()),
            ),
            group: Some(activity.group.map_or("".to_string(), |id| id.to_string())),
            ty: activity.ty.to_string(),
            create_by: activity.create_by.to_string(),
            uo: Some(activity.uo.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
            dl: activity
                .dl
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            co: activity
                .co
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
        }
    }

    pub fn put(ty: ActivityModelPut) -> Document {
        let mut doc = Document::new();

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
            }
        };

        insert_if_some(
            "ty",
            ty.ty
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        insert_if_some(
            "group",
            ty.group
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        insert_if_some(
            "class_id",
            ty.class_id
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        insert_if_some(
            "dl",
            ty.dl
                .map(|date| bson::Bson::DateTime(DateTime::parse_rfc3339_str(&date).unwrap())),
        );

        insert_if_some("act", ty.act.map(bson::Bson::String));

        doc
    }
}
