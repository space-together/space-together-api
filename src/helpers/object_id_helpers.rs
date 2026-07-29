use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{models::id_model::IdType, utils::object_id::ObjectId};

pub fn serialize<S>(oid: &Option<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match oid {
        Some(oid) => serializer.serialize_str(&oid.to_hex()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ObjectId>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    parse_optional_object_id_value(value).map_err(serde::de::Error::custom)
}

pub fn serialize_vec<S>(oids: &[ObjectId], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ids: Vec<String> = oids.iter().map(|oid| oid.to_hex()).collect();
    ids.serialize(serializer)
}

pub fn deserialize_vec<'de, D>(deserializer: D) -> Result<Vec<ObjectId>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    parse_object_id_array_value(value).map_err(serde::de::Error::custom)
}

pub fn serialize_opt_vec<S>(oids: &Option<Vec<ObjectId>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match oids {
        Some(ids) => serialize_vec(ids, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_opt_vec<'de, D>(deserializer: D) -> Result<Option<Vec<ObjectId>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(None),
        other => parse_object_id_array_value(other)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

pub fn serialize_oid<S>(oid: &ObjectId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&oid.to_hex())
}

pub fn deserialize_oid<'de, D>(deserializer: D) -> Result<ObjectId, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    parse_required_object_id_value(value).map_err(serde::de::Error::custom)
}

pub fn serialize_vec_oid<S>(oids: &[ObjectId], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serialize_vec(oids, serializer)
}

pub fn deserialize_vec_oid<'de, D>(deserializer: D) -> Result<Vec<ObjectId>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_vec(deserializer)
}

pub fn parse_object_id(id: &IdType) -> Result<ObjectId, String> {
    match id {
        IdType::String(id_str) => ObjectId::parse_str(id_str).map_err(|err| err.to_string()),
        IdType::ObjectId(oid) => Ok(oid.clone()),
    }
}

fn parse_optional_object_id_value(value: Value) -> Result<Option<ObjectId>, String> {
    match value {
        Value::Null => Ok(None),
        other => parse_required_object_id_value(other).map(Some),
    }
}

fn parse_required_object_id_value(value: Value) -> Result<ObjectId, String> {
    match value {
        Value::String(id) => ObjectId::parse_str(&id).map_err(|err| err.to_string()),
        Value::Object(mut map) => match map.remove("$oid") {
            Some(Value::String(id)) => ObjectId::parse_str(&id).map_err(|err| err.to_string()),
            _ => Err("Missing $oid field".to_string()),
        },
        _ => Err("Expected ObjectId-compatible string".to_string()),
    }
}

fn parse_object_id_array_value(value: Value) -> Result<Vec<ObjectId>, String> {
    match value {
        Value::Array(items) => items
            .into_iter()
            .map(parse_required_object_id_value)
            .collect(),
        Value::Null => Ok(vec![]),
        _ => Err("Expected ObjectId-compatible array".to_string()),
    }
}
