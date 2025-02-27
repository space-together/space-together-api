use serde::{Deserialize, Serialize};

use crate::libs::types::fields_types::{DateType, IdType};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SectorModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<IdType>,
    pub education_id: Option<IdType>,
    pub username: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub symbol_id: Option<IdType>,
    pub create_on: Option<DateType>,
    pub updated_on: Option<DateType>,
}
