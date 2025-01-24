use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClassRoomModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: Option<String>,
    pub sector_id: Option<ObjectId>,
    pub trade_id: Option<ObjectId>,
    pub symbol_id: Option<ObjectId>,
    pub class_room_type_id: Option<ObjectId>,
    pub description: Option<String>,
    pub created_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClassRoomModelGet {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub description: Option<String>,
    pub sector: Option<String>,
    pub trade: Option<String>,
    pub class_room_type: Option<String>,
    pub created_on: String,
    pub updated_on: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClassRoomModelNew {
    pub name: String,
    pub username: Option<String>,
    pub description: Option<String>,
    pub sector: Option<String>,
    pub trade: Option<String>,
    pub class_room_type: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClassRoomModelPut {
    pub name: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub sector: Option<String>,
    pub trade: Option<String>,
    pub class_room_type: Option<String>,
    pub symbol: Option<String>,
}

impl ClassRoomModel {
    pub fn new(class_room: ClassRoomModelNew) -> Self {
        ClassRoomModel {
            id: None,
            name: class_room.name,
            username: class_room.username,
            class_room_type_id: class_room.class_room_type.map(|id| {
                ObjectId::from_str(&id).expect("can change class room id into object is")
            }),
            sector_id: class_room.sector.map(|id| {
                ObjectId::from_str(&id).expect("can change class room id into object is")
            }),
            trade_id: class_room.trade.map(|id| {
                ObjectId::from_str(&id).expect("can change class room id into object is")
            }),
            description: class_room.description,
            symbol_id: class_room.symbol.map(|id| ObjectId::from_str(&id).unwrap()),
            created_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(class_room: Self) -> ClassRoomModelGet {
        ClassRoomModelGet {
            id: class_room.id.map_or("".to_string(), |id| id.to_string()),
            class_room_type: class_room.class_room_type_id.map(|id| id.to_string()),
            sector: class_room.sector_id.map(|id| id.to_string()),
            trade: class_room.trade_id.map(|id| id.to_string()),
            name: class_room.name,
            username: class_room.username,
            description: class_room.description,
            symbol: class_room.symbol_id.map(|id| id.to_string()),
            created_on: class_room
                .created_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: Some(class_room.updated_on.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }

    pub fn put(class_room: ClassRoomModelPut) -> Document {
        let mut doc = Document::new();
        let mut is_update = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
                is_update = true;
            }
        };

        insert_if_some("name", class_room.name.map(bson::Bson::String));
        insert_if_some("username", class_room.username.map(bson::Bson::String));
        insert_if_some(
            "description",
            class_room.description.map(bson::Bson::String),
        );
        insert_if_some(
            "symbol_id",
            class_room
                .symbol
                .and_then(|id| ObjectId::from_str(&id).ok())
                .map(bson::Bson::ObjectId),
        );

        insert_if_some(
            "sector_id",
            class_room
                .sector
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );
        insert_if_some(
            "trade_id",
            class_room
                .trade
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        insert_if_some(
            "class_room_type_id",
            class_room
                .class_room_type
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        if is_update {
            doc.insert("updated_on", bson::Bson::DateTime(DateTime::now()));
        }

        doc
    }
}
