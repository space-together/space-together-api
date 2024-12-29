use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::{
    libs::functions::characters_fn::{generate_code, generate_username},
    models::other_model::{
        address_model::AddressModel,
        contact_model::{ContactModel, SocialMediaModel},
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolMember {
    id: ObjectId,
    disable: bool,
    is_pending: bool,
    added_on: DateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolMemberGet {
    id: String,
    disable: bool,
    is_pending: bool,
    added_on: String,
}

impl SchoolMember {
    fn format(member: Self) -> SchoolMemberGet {
        SchoolMemberGet {
            id: member.id.to_string(),
            disable: member.disable,
            is_pending: member.is_pending,
            added_on: member
                .added_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>, // Unique identifier for the school
    pub owner: ObjectId,                        // user how create school
    pub username: String,                       // school username
    pub name: String,                           // School name
    pub code: String,                           // Unique school code
    pub description: Option<String>,            // Brief description of the school
    pub address: AddressModel,                  // School address
    pub contact: ContactModel,                  // contact
    pub website: Option<String>,                // School website URL
    pub social_media: Option<SocialMediaModel>, // Links to social media accounts
    pub principal_id: Option<ObjectId>,         // Reference to the principal (user ID)
    pub staff_ids: Option<Vec<SchoolMember>>,   // List of staff members (user IDs)
    pub student_ids: Option<Vec<SchoolMember>>, // List of students (user IDs)
    pub teacher_ids: Option<Vec<SchoolMember>>, // List of teacher (user IDs)
    pub classes: Option<Vec<ObjectId>>,         // List of classes in the school
    pub logo_uri: Option<String>,               // URI for the school logo
    pub is_active: bool,                        // Whether the school is operational
    pub created_on: DateTime,                   // Record creation date
    pub updated_on: Option<DateTime>,           // Record last updated date
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolModelGet {
    pub id: String,                                // Unique identifier for the school
    pub owner: String,                             // user how create school
    pub username: String,                          // school username
    pub name: String,                              // School name
    pub code: String,                              // Unique school code
    pub description: Option<String>,               // Brief description of the school
    pub address: AddressModel,                     // School address
    pub contact: ContactModel,                     // contact
    pub website: Option<String>,                   // School website URL
    pub social_media: Option<SocialMediaModel>,    // Links to social media accounts
    pub principal_id: Option<String>,              // Reference to the principal (user ID)
    pub staff_ids: Option<Vec<SchoolMemberGet>>,   // List of staff members (user IDs)
    pub student_ids: Option<Vec<SchoolMemberGet>>, // List of students (user IDs)
    pub teacher_ids: Option<Vec<SchoolMemberGet>>, // List of teacher (user IDs)
    pub classes: Option<Vec<String>>,              // List of classes in the school
    pub logo_uri: Option<String>,                  // URI for the school logo
    pub is_active: bool,                           // Whether the school is operational
    pub created_on: String,                        // Record creation date
    pub updated_on: Option<String>,                // Record last updated date
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolModelNew {
    pub owner: String,               // user how create school
    pub name: String,                // School name
    pub description: Option<String>, // Brief description of the school
    pub address: AddressModel,       // School address
    pub contact: ContactModel,       // contact
    pub website: Option<String>,     // School website URL
    pub logo_uri: Option<String>,    // URI for the school logo
}

impl SchoolModel {
    pub fn new(school: SchoolModelNew) -> Self {
        SchoolModel {
            id: None,
            owner: ObjectId::from_str(&school.owner).unwrap(),
            username: generate_username(&school.name),
            name: school.name,
            description: school.description,
            code: generate_code(),
            address: school.address,
            contact: school.contact,
            website: school.website,
            social_media: None,
            principal_id: None,
            staff_ids: None,
            student_ids: None,
            teacher_ids: None,
            classes: None,
            logo_uri: school.logo_uri,
            is_active: false,
            created_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(school: Self) -> SchoolModelGet {
        SchoolModelGet {
            id: school.id.map_or("".to_string(), |id| id.to_string()),
            code: school.code,
            owner: school.owner.to_string(),
            name: school.name,
            username: school.username,
            description: school.description,
            address: school.address,
            classes: school
                .classes
                .map(|classes| classes.into_iter().map(|id| id.to_string()).collect()),
            social_media: school.social_media,
            staff_ids: school
                .staff_ids
                .map(|staffs| staffs.into_iter().map(SchoolMember::format).collect()),
            student_ids: school
                .student_ids
                .map(|users| users.into_iter().map(SchoolMember::format).collect()),
            is_active: school.is_active,
            contact: school.contact,
            website: school.website,
            principal_id: school.principal_id.map(|id| id.to_string()),
            teacher_ids: school
                .teacher_ids
                .map(|teachers| teachers.into_iter().map(SchoolMember::format).collect()),
            logo_uri: school.logo_uri,
            created_on: school
                .created_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: school
                .updated_on
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or("".to_string())),
        }
    }
}
