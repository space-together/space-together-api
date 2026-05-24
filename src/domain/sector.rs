use crate::{
    helpers::{date_time_helpers, object_id_helpers},
    make_partial,
};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sector {
    #[serde(
        rename = "_id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    pub name: String,
    pub logo: Option<String>,
    pub logo_id: Option<String>,
    pub username: String,
    pub description: Option<String>,
    pub curriculum: Option<(i32, i32)>, // start-end years
    pub country: String,
    pub r#type: String, // global, international, local
    pub disable: Option<bool>,

    #[serde(
        default,
        serialize_with = "date_time_helpers::optional_chrono_datetime::serialize",
        deserialize_with = "date_time_helpers::optional_chrono_datetime::deserialize"
    )]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(
        default,
        serialize_with = "date_time_helpers::optional_chrono_datetime::serialize",
        deserialize_with = "date_time_helpers::optional_chrono_datetime::deserialize"
    )]
    pub updated_at: Option<DateTime<Utc>>,
} => SectorPartial
}
