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
pub struct SchoolModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>, // Unique identifier for the school
    pub creator_id: ObjectId, // User who created the school

    // Basic Information
    pub username: String,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub school_type: String,           // Public | Private | International
    pub curriculum: Vec<String>,       // ["REB", "TVET"]
    pub education_levels: Vec<String>, // ["Primary", "Secondary", "TVET"]
    pub school_members: String,        // Boys only | Girls only | Mixed
    pub accreditation_number: String,
    pub affiliation: String, // Government | Religious | NGO | Independent

    // Location & Contact
    pub address: AddressModel,
    pub contact: ContactModel,
    pub website: Option<String>,
    pub social_media: Option<SocialMediaModel>,

    // Student Information
    pub student_capacity: u32,
    pub current_students: u32,
    pub grading_system: Vec<String>,
    pub uniform_required: bool,
    pub uniform_description: Option<String>,
    pub attendance_system: String, // Online | Manual
    pub scholarships_available: bool,

    // Facilities
    pub classrooms: u32,
    pub library: bool,
    pub labs: Vec<String>, // ["Science", "Computer", "Engineering"]
    pub sports_extracurricular: Vec<String>, // ["Football", "Debate", "Coding Club"]
    pub online_classes: bool,

    // Legal Information
    pub registration_number: String,
    pub accreditation_body: String,
    pub school_motto: Option<String>,
    pub logo_uri: Option<String>,

    // Meta Data
    pub is_active: bool,
    pub created_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolModelGet {
    pub id: String,
    pub creator_id: String,
    pub username: String,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub school_type: String,
    pub curriculum: Vec<String>,
    pub education_levels: Vec<String>,
    pub school_members: String,
    pub accreditation_number: String,
    pub affiliation: String,

    // Location & Contact
    pub address: AddressModel,
    pub contact: ContactModel,
    pub website: Option<String>,
    pub social_media: Option<SocialMediaModel>,

    // Administration
    pub principal_id: Option<String>,
    pub classes: Option<Vec<String>>,

    // Student Information
    pub student_capacity: u32,
    pub current_students: u32,
    pub grading_system: Vec<String>,
    pub uniform_required: bool,
    pub uniform_description: Option<String>,
    pub attendance_system: String,
    pub scholarships_available: bool,

    // Facilities
    pub classrooms: u32,
    pub library: bool,
    pub labs: Vec<String>,
    pub sports_extracurricular: Vec<String>,
    pub online_classes: bool,

    // Legal Information
    pub registration_number: String,
    pub accreditation_body: String,
    pub school_motto: Option<String>,
    pub logo_uri: Option<String>,

    // Meta Data
    pub is_active: bool,
    pub created_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolModelNew {
    pub creator_id: String,
    pub name: String,
    pub description: Option<String>,
    pub school_type: String,
    pub curriculum: Vec<String>,
    pub education_levels: Vec<String>,
    pub school_members: String,
    pub accreditation_number: String,
    pub affiliation: String,
    pub address: AddressModel,
    pub contact: ContactModel,
    pub website: Option<String>,
    pub logo_uri: Option<String>,
}

impl SchoolModel {
    pub fn new(school: SchoolModelNew) -> Self {
        SchoolModel {
            id: None,
            creator_id: ObjectId::from_str(&school.creator_id).unwrap(),
            username: generate_username(&school.name),
            name: school.name,
            description: school.description,
            code: generate_code(),
            school_type: school.school_type,
            curriculum: school.curriculum,
            education_levels: school.education_levels,
            school_members: school.school_members,
            accreditation_number: school.accreditation_number,
            affiliation: school.affiliation,
            address: school.address,
            contact: school.contact,
            website: school.website,
            social_media: None,
            student_capacity: 0,
            current_students: 0,
            grading_system: vec![],
            uniform_required: false,
            uniform_description: None,
            attendance_system: "Manual".to_string(),
            scholarships_available: false,
            classrooms: 0,
            library: false,
            labs: vec![],
            sports_extracurricular: vec![],
            online_classes: false,
            registration_number: "".to_string(),
            accreditation_body: "".to_string(),
            school_motto: None,
            logo_uri: None,
            is_active: false,
            created_on: DateTime::now(),
            updated_on: None,
        }
    }
}

// school member

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolMemberRole {
    Student,
    Teacher,
    Parent,
    Headmaster,
    Principal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolMemberModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub school_id: ObjectId,
    pub role: SchoolMemberRole,
    pub email: Option<String>,
    pub disable: bool,
    pub is_pending: bool,
    pub added_on: DateTime,
    pub create_at: DateTime,
    pub update_at: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolMemberModelGet {
    pub id: String,
    pub disable: bool,
    pub is_pending: bool,
    pub user_id: String,
    pub added_on: String,
    pub email: Option<String>,
    pub role: SchoolMemberRole,
    pub school_id: String,
    pub create_at: String,
    pub update_at: Option<String>,
}

impl SchoolMemberModel {
    fn format(member: Self) -> SchoolMemberModelGet {
        SchoolMemberModelGet {
            id: member.id.map_or("".to_string(), |i| i.to_string()),
            disable: member.disable,
            is_pending: member.is_pending,
            role: member.role,
            school_id: member.school_id.to_string(),
            user_id: member.user_id.to_string(),
            email: member.email,
            create_at: member
                .create_at
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            update_at: member
                .update_at
                .map(|d| d.try_to_rfc3339_string().unwrap_or("".to_string())),
            added_on: member
                .added_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
        }
    }
}
