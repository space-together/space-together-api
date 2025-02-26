use serde::{Deserialize, Serialize};

use crate::libs::types::fields_types::{DateType, IdType};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EducationSchema {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<IdType>,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub symbol: Option<IdType>,
    pub roles: Option<Vec<String>>,
    pub disabled: Option<bool>,
    pub created_at: Option<DateType>,
    pub updated_at: Option<DateType>,
}
