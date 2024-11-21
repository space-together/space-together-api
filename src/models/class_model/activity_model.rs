use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::error::class_error::activities_error::{ActivitiesErr, ActivitiesResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivityModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub ty: ObjectId,         // Activity type
    pub ow: ObjectId,         // create by
    pub gr: Option<ObjectId>, // group
    pub cl: Option<ObjectId>, // class id
    pub act: String,          // activity
    pub dl: DateTime,         // died line
    pub co: DateTime,         // created at
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivityModelGet {
    pub id: String,
    pub ty: String,         // Activity type
    pub ow: String,         // create by
    pub act: String,        // activity
    pub dl: String,         // died line
    pub gr: Option<String>, // group
    pub cl: Option<String>, // class id
    pub co: String,         // created at
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivityModelNew {
    pub ty: String,
    pub cl: Option<String>,
    pub gr: Option<String>,
    pub ow: String,
    pub act: String,
    pub dl: String,
}

impl ActivityModel {
    pub fn new(activity: ActivityModelNew) -> ActivitiesResult<Self> {
        let ty = ObjectId::from_str(&activity.ty).map_err(|_| ActivitiesErr::Invalid)?;
        let ow = ObjectId::from_str(&activity.ow).map_err(|_| ActivitiesErr::Invalid)?;
        let cl = match &activity.cl {
            Some(cl) => Some(ObjectId::from_str(cl).map_err(|_| ActivitiesErr::Invalid)?),
            None => None,
        };
        let gr = match &activity.gr {
            Some(gr) => Some(ObjectId::from_str(gr).map_err(|_| ActivitiesErr::Invalid)?),
            None => None,
        };
        let dl = DateTime::parse_rfc3339_str(&activity.dl)
            .map_err(|_| ActivitiesErr::InvalidDateTime)?;

        Ok(ActivityModel {
            id: None,
            ow,
            ty,
            cl,
            gr,
            act: activity.act,
            dl,
            co: DateTime::now(),
        })
    }

    pub fn format(activity: ActivityModel) -> ActivityModelGet {
        ActivityModelGet {
            id: activity.id.map_or("".to_string(), |id| id.to_string()),
            act: activity.act,
            cl: Some(activity.cl.map_or("".to_string(), |id| id.to_string())),
            gr: Some(activity.gr.map_or("".to_string(), |id| id.to_string())),
            ty: activity.ty.to_string(),
            ow: activity.ow.to_string(),
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
}
