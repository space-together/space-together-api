use serde::{Deserialize, Serialize};

use crate::libs::types::fields_types::{DateType, IdType};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClassRoomSchema {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<IdType>,
    pub name: String,
    pub username: Option<String>,
    pub sector_id: Option<IdType>,
    pub trade_id: Option<IdType>,
    pub symbol_id: Option<IdType>,
    pub class_room_type_id: Option<IdType>,
    pub description: Option<String>,
    pub created_at: Option<DateType>,
    pub updated_at: Option<DateType>,
}
