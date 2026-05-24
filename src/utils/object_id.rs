use mongodb::bson::oid::ObjectId as MongoObjectId;

use crate::errors::AppError;

pub type ObjectId = MongoObjectId;

pub fn parse_object_id_value(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|e| AppError {
        message: format!("Invalid ObjectId-compatible ID: {}", e),
    })
}
