use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TradeModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub sector_id: Option<ObjectId>,
    pub max_classes: i32,
    pub min_classes: i32,
    pub symbol: Option<ObjectId>,
    pub display: Option<bool>,
    pub create_at: DateTime,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TradeModelGet {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub description: Option<String>,
    pub max_classes: i32,
    pub min_classes: i32,
    pub sector: Option<String>,
    pub symbol: Option<String>,
    pub create_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TradeModelNew {
    pub name: String,
    pub username: Option<String>,
    pub sector: Option<String>,
    pub description: Option<String>,
    pub max_classes: i32,
    pub min_classes: i32,
    pub symbol: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TradeModelPut {
    pub name: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub sector: Option<String>,
    pub max_classes: Option<i32>,
    pub min_classes: Option<i32>,
    pub symbol: Option<String>,
}

impl TradeModel {
    pub fn new(trade: TradeModelNew) -> Self {
        TradeModel {
            id: None,
            name: trade.name,
            username: trade.username.unwrap_or_default(),
            description: trade.description,
            sector_id: trade.sector.map(|id| ObjectId::from_str(&id).unwrap()),
            max_classes: trade.max_classes,
            min_classes: trade.min_classes,
            symbol: trade.symbol.map(|id| ObjectId::from_str(&id).unwrap()),
            display: None,
            create_at: DateTime::now(),
            updated_at: None,
        }
    }

    pub fn format(trade: Self) -> TradeModelGet {
        TradeModelGet {
            id: trade.id.map_or("".to_string(), |id| id.to_string()),
            name: trade.name,
            username: Some(trade.username),
            description: trade.description,
            max_classes: trade.max_classes,
            min_classes: trade.min_classes,
            sector: trade.sector_id.map(|id| id.to_string()),
            symbol: trade.symbol.map(|id| id.to_string()),
            create_at: trade
                .create_at
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_at: trade
                .updated_at
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or("".to_string())),
        }
    }

    pub fn put(trade: TradeModelPut) -> Document {
        let mut set_doc = Document::new();
        let mut is_updated = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                set_doc.insert(key, v);
                is_updated = true;
            }
        };

        insert_if_some("name", trade.name.map(bson::Bson::String));
        insert_if_some("username", trade.username.map(bson::Bson::String));
        insert_if_some("max_classes", trade.max_classes.map(bson::Bson::Int32));
        insert_if_some("min_classes", trade.min_classes.map(bson::Bson::Int32));
        insert_if_some(
            "sector_id",
            trade
                .sector
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );
        insert_if_some("description", trade.description.map(bson::Bson::String));
        insert_if_some(
            "symbol_id",
            trade
                .symbol
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        if is_updated {
            set_doc.insert("updated_at", bson::Bson::DateTime(DateTime::now()));
        }

        set_doc
    }
}
