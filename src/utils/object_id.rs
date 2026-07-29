use bson::oid::ObjectId as BsonObjectId;

use crate::errors::AppError;

pub type ObjectId = BsonObjectId;

pub fn parse_object_id_value(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|e| AppError {
        message: format!("Invalid ObjectId-compatible ID: {}", e),
    })
}
