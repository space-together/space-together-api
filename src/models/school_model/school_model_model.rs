use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::models::other_model::address_model::AddressModel;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>, // Unique identifier for the school
    pub name: String,                         // School name
    pub code: String,                         // Unique school code
    pub description: Option<String>,          // Brief description of the school
    pub accreditation_status: Option<String>, // Accreditation status of the school
    pub address: AddressModel,                // School address
    pub contact_email: String,                // Contact email of the school
    pub contact_phone: Option<String>,        // Contact phone number of the school
    pub principal_id: Option<ObjectId>,       // Reference to the principal (user ID)
    pub staff_ids: Option<Vec<ObjectId>>,     // List of staff members (user IDs)
    pub student_ids: Option<Vec<ObjectId>>,   // List of students (user IDs)
    pub classes: Option<Vec<ObjectId>>,       // List of classes in the school
    pub facilities: Option<Vec<String>>,      // List of facilities (e.g., Library, Labs)
    pub logo_uri: Option<String>,             // URI for the school logo
    pub documents: Option<Vec<String>>,       // URIs for administrative documents
    pub registration_date: Option<DateTime>,  // School's registration date
    pub is_active: bool,                      // Whether the school is operational
    pub created_on: DateTime,                 // Record creation date
    pub updated_on: Option<DateTime>,         // Record last updated date
}
