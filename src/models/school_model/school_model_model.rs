use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

use crate::{
    libs::functions::characters_fn::{generate_code, generate_username},
    models::other_model::{
        address_model::AddressModel,
        contact_model::{ContactModel, SocialMediaModel},
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolType {
    Public,
    Private,
    International,
}

#[allow(clippy::inherent_to_string)]
impl SchoolType {
    pub(crate) fn to_string(&self) -> String {
        match self {
            SchoolType::Public => "Public".to_string(),
            SchoolType::Private => "Private".to_string(),
            SchoolType::International => "International".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolMembersType {
    BoysOnly,
    GirlsOnly,
    Mixed,
}

#[allow(clippy::inherent_to_string)]
impl SchoolMembersType {
    pub(crate) fn to_string(&self) -> String {
        match self {
            SchoolMembersType::BoysOnly => "BoysOnly".to_string(),
            SchoolMembersType::GirlsOnly => "GirlsOnly".to_string(),
            SchoolMembersType::Mixed => "Mixed".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolAffiliation {
    Government,
    Religious,
    NGO,
    Independent,
}

#[allow(clippy::inherent_to_string)]
impl SchoolAffiliation {
    pub(crate) fn to_string(&self) -> String {
        match self {
            SchoolAffiliation::Government => "Government".to_string(),
            SchoolAffiliation::Religious => "Religious".to_string(),
            SchoolAffiliation::NGO => "NGO".to_string(),
            SchoolAffiliation::Independent => "Independent".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolCurriculum {
    REB,
    TVET,
    IB,    // International Baccalaureate
    IGCSE, // International General Certificate of Secondary Education
}

#[allow(clippy::inherent_to_string)]
impl SchoolCurriculum {
    pub(crate) fn to_string(&self) -> String {
        match self {
            SchoolCurriculum::REB => "REB".to_string(),
            SchoolCurriculum::TVET => "TVET".to_string(),
            SchoolCurriculum::IB => "IB".to_string(),
            SchoolCurriculum::IGCSE => "IGCSE".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolEducationLevel {
    Primary,
    Secondary,
    TVET,
    HigherEducation,
}

#[allow(clippy::inherent_to_string)]
impl SchoolEducationLevel {
    pub(crate) fn to_string(&self) -> String {
        match self {
            SchoolEducationLevel::Primary => "Primary".to_string(),
            SchoolEducationLevel::Secondary => "Secondary".to_string(),
            SchoolEducationLevel::TVET => "TVET".to_string(),
            SchoolEducationLevel::HigherEducation => "HigherEducation".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolAttendanceSystem {
    Online,
    Manual,
}

#[allow(clippy::inherent_to_string)]
impl SchoolAttendanceSystem {
    pub(crate) fn to_string(&self) -> String {
        match self {
            SchoolAttendanceSystem::Online => "Online".to_string(),
            SchoolAttendanceSystem::Manual => "Manual".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolGradingSystem {
    AF,
    Percentage,
    GPA,
}

#[allow(clippy::inherent_to_string)]
impl SchoolGradingSystem {
    pub(crate) fn to_string(&self) -> String {
        match self {
            SchoolGradingSystem::AF => "AF".to_string(),
            SchoolGradingSystem::Percentage => "Percentage".to_string(),
            SchoolGradingSystem::GPA => "GPA".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolFacilities {
    Classrooms,
    Library,
    Labs,
    SportsExtracurricular,
    OnlineClasses,
}

#[allow(clippy::inherent_to_string)]
impl SchoolFacilities {
    pub(crate) fn to_string(&self) -> String {
        match self {
            SchoolFacilities::Classrooms => "Classrooms".to_string(),
            SchoolFacilities::Library => "Library".to_string(),
            SchoolFacilities::Labs => "Labs".to_string(),
            SchoolFacilities::OnlineClasses => "OnlineClasses".to_string(),
            SchoolFacilities::SportsExtracurricular => "SportsExtracurricular".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolLabs {
    Science,
    Computer,
    Engineering,
    Language,
}

#[allow(clippy::inherent_to_string)]
impl SchoolLabs {
    pub(crate) fn to_string(&self) -> String {
        match self {
            SchoolLabs::Science => "Science".to_string(),
            SchoolLabs::Computer => "Computer".to_string(),
            SchoolLabs::Engineering => "Engineering".to_string(),
            SchoolLabs::Language => "Language".to_string(),
        }
    }
}
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
    pub school_type: SchoolType, // Public | Private | International
    pub curriculum: Vec<SchoolCurriculum>, // ["REB", "TVET"]
    pub education_levels: Vec<SchoolEducationLevel>, // ["Primary", "Secondary", "TVET"]
    pub school_members: SchoolMembersType, // Boys only | Girls only | Mixed
    pub accreditation_number: String,
    pub affiliation: SchoolAffiliation, // Government | Religious | NGO | Independent

    // Location & Contact
    pub address: AddressModel,
    pub contact: ContactModel,
    pub website: Option<String>,
    pub social_media: Option<SocialMediaModel>,

    // Student Information
    pub student_capacity: u32,
    pub current_students: u32,
    pub grading_system: Vec<SchoolGradingSystem>,
    pub uniform_required: bool,
    pub uniform_description: Option<String>,
    pub attendance_system: SchoolAttendanceSystem, // Online | Manual
    pub scholarships_available: bool,

    // Facilities
    pub classrooms: u32,
    pub library: bool,
    pub labs: Vec<SchoolLabs>, // ["Science", "Computer", "Engineering"]
    pub sports_extracurricular: Vec<String>, // ["Football", "Debate", "Coding Club"]
    pub online_classes: bool,

    // Legal Information
    pub registration_number: String,
    pub accreditation_body: String,
    pub school_motto: Option<String>,
    pub logo: Option<String>,

    // Meta Data
    pub is_active: bool,
    pub created_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolModelGet {
    pub id: String,         // Unique identifier for the school
    pub creator_id: String, // User who created the school

    // Basic Information
    pub username: String,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub school_type: SchoolType, // Public | Private | International
    pub curriculum: Vec<SchoolCurriculum>, // ["REB", "TVET"]
    pub education_levels: Vec<SchoolEducationLevel>, // ["Primary", "Secondary", "TVET"]
    pub school_members: SchoolMembersType, // Boys only | Girls only | Mixed
    pub accreditation_number: String,
    pub affiliation: SchoolAffiliation, // Government | Religious | NGO | Independent

    // Location & Contact
    pub address: Option<AddressModel>,
    pub contact: Option<ContactModel>,
    pub website: Option<String>,
    pub social_media: Option<SocialMediaModel>,

    // Student Information
    pub student_capacity: Option<u32>,
    pub current_students: Option<u32>,
    pub grading_system: Vec<SchoolGradingSystem>,
    pub uniform_required: Option<bool>,
    pub uniform_description: Option<String>,
    pub attendance_system: SchoolAttendanceSystem, // Online | Manual
    pub scholarships_available: Option<bool>,

    // Facilities
    pub classrooms: Option<u32>,
    pub library: Option<bool>,
    pub labs: Vec<SchoolLabs>, // ["Science", "Computer", "Engineering"]
    pub sports_extracurricular: Option<Vec<String>>, // ["Football", "Debate", "Coding Club"]
    pub online_classes: Option<bool>,

    // Legal Information
    pub registration_number: Option<String>,
    pub accreditation_body: Option<String>,
    pub school_motto: Option<String>,
    pub logo: Option<String>,

    // Meta Data
    pub is_active: bool,
    pub created_on: String,
    pub updated_on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolModelPut {
    pub creator_id: Option<String>, // User who created the school

    // Basic Information
    pub username: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub school_type: Option<SchoolType>, // Public | Private | International
    pub curriculum: Option<Vec<SchoolCurriculum>>, // ["REB", "TVET"]
    pub education_levels: Option<Vec<SchoolEducationLevel>>, // ["Primary", "Secondary", "TVET"]
    pub school_members: Option<SchoolMembersType>, // Boys only | Girls only | Mixed
    pub accreditation_number: Option<String>,
    pub affiliation: Option<SchoolAffiliation>, // Government | Religious | NGO | Independent

    // Location & Contact
    pub address: Option<AddressModel>,
    pub contact: Option<ContactModel>,
    pub website: Option<String>,
    pub social_media: Option<SocialMediaModel>,

    // Student Information
    pub student_capacity: Option<u32>,
    pub current_students: Option<u32>,
    pub grading_system: Option<Vec<SchoolGradingSystem>>,
    pub uniform_required: Option<bool>,
    pub uniform_description: Option<String>,
    pub attendance_system: Option<SchoolAttendanceSystem>, // Online | Manual
    pub scholarships_available: Option<bool>,

    // Facilities
    pub classrooms: Option<u32>,
    pub library: Option<bool>,
    pub labs: Option<Vec<SchoolLabs>>, // ["Science", "Computer", "Engineering"]
    pub sports_extracurricular: Option<Vec<String>>, // ["Football", "Debate", "Coding Club"]
    pub online_classes: Option<bool>,

    // Legal Information
    pub registration_number: Option<String>,
    pub accreditation_body: Option<String>,
    pub school_motto: Option<String>,
    pub logo: Option<String>,

    // Meta Data
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolModelNew {
    pub creator_id: String,
    pub name: String,
    pub description: Option<String>,
    pub school_type: SchoolType, // Public | Private | International
    pub curriculum: Vec<SchoolCurriculum>,
    pub education_levels: Vec<SchoolEducationLevel>, // ["Primary", "Secondary", "TVET"]
    pub school_members: SchoolMembersType,
    pub accreditation_number: String,
    pub affiliation: SchoolAffiliation, // Government | Religious | NGO | Independent
    pub address: AddressModel,
    pub contact: ContactModel,
    pub website: Option<String>,
    pub logo: Option<String>,
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
            attendance_system: SchoolAttendanceSystem::Manual,
            scholarships_available: false,
            classrooms: 0,
            library: false,
            labs: vec![],
            sports_extracurricular: vec![],
            online_classes: false,
            registration_number: "".to_string(),
            accreditation_body: "".to_string(),
            school_motto: None,
            logo: None,
            is_active: false,
            created_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(&self) -> SchoolModelGet {
        SchoolModelGet {
            id: self.id.as_ref().map_or("".to_string(), |i| i.to_string()),
            creator_id: self.creator_id.to_string(),
            username: self.username.clone(),
            name: self.name.clone(),
            code: self.code.clone(),
            description: self.description.clone(),
            school_type: self.school_type.clone(),
            curriculum: self.curriculum.clone(),
            education_levels: self.education_levels.clone(),
            school_members: self.school_members.clone(),
            accreditation_number: self.accreditation_number.clone(),
            affiliation: self.affiliation.clone(),
            address: Some(self.address.clone()),
            contact: Some(self.contact.clone()),
            website: self.website.clone(),
            social_media: self.social_media.clone(),
            student_capacity: Some(self.student_capacity),
            current_students: Some(self.current_students),
            grading_system: self.grading_system.clone(),
            uniform_required: Some(self.uniform_required),
            uniform_description: self.uniform_description.clone(),
            attendance_system: self.attendance_system.clone(),
            scholarships_available: Some(self.scholarships_available),
            classrooms: Some(self.classrooms),
            library: Some(self.library),
            labs: self.labs.clone(),
            sports_extracurricular: Some(self.sports_extracurricular.clone()),
            online_classes: Some(self.online_classes),
            registration_number: Some(self.registration_number.clone()),
            accreditation_body: Some(self.accreditation_body.clone()),
            school_motto: self.school_motto.clone(),
            logo: self.logo.clone(),
            is_active: self.is_active,
            created_on: self
                .created_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: self
                .updated_on
                .map(|d| d.try_to_rfc3339_string().unwrap_or("".to_string())),
        }
    }

    pub fn put(school: SchoolModelPut) -> Document {
        let mut set_doc = Document::new();
        let mut is_updated = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                set_doc.insert(key, v);
                is_updated = true;
            }
        };
        insert_if_some(
            "creator_id",
            school
                .creator_id
                .map(|i| bson::Bson::ObjectId(ObjectId::from_str(&i).unwrap())),
        );
        insert_if_some("name", school.name.map(bson::Bson::String));
        insert_if_some("username", school.username.map(bson::Bson::String));
        insert_if_some("code", school.code.map(bson::Bson::String));
        insert_if_some("description", school.description.map(bson::Bson::String));
        insert_if_some(
            "school_type",
            school
                .school_type
                .map(|v| bson::Bson::String(v.to_string())),
        );
        insert_if_some(
            "curriculum",
            school
                .curriculum
                .map(|v| bson::to_bson(&v).unwrap())
                .map(|v| bson::Bson::Array(v.as_array().unwrap().to_vec())),
        );

        insert_if_some(
            "education_levels",
            school
                .education_levels
                .map(|v| bson::to_bson(&v).unwrap())
                .map(|v| bson::Bson::Array(v.as_array().unwrap().to_vec())),
        );
        insert_if_some(
            "school_members",
            school
                .school_members
                .map(|v| bson::Bson::String(v.to_string())),
        );

        insert_if_some(
            "accreditation_number",
            school.accreditation_number.map(bson::Bson::String),
        );
        insert_if_some(
            "affiliation",
            school
                .affiliation
                .map(|v| bson::Bson::String(v.to_string())),
        );
        insert_if_some(
            "address",
            school.address.map(|v| bson::to_bson(&v).unwrap()),
        );
        insert_if_some(
            "contact",
            school.contact.map(|v| bson::to_bson(&v).unwrap()),
        );
        insert_if_some(
            "social_media",
            school.social_media.map(|v| bson::to_bson(&v).unwrap()),
        );
        insert_if_some("website", school.website.map(bson::Bson::String));
        insert_if_some(
            "student_capacity",
            school.student_capacity.map(|v| bson::Bson::Int32(v as i32)),
        );
        insert_if_some(
            "current_students",
            school.current_students.map(|v| bson::Bson::Int32(v as i32)),
        );
        insert_if_some(
            "grading_system",
            school
                .grading_system
                .map(|v| bson::to_bson(&v).unwrap())
                .map(|v| bson::Bson::Array(v.as_array().unwrap().to_vec())),
        );
        insert_if_some(
            "uniform_required",
            school.uniform_required.map(bson::Bson::Boolean),
        );
        insert_if_some(
            "uniform_description",
            school.uniform_description.map(bson::Bson::String),
        );

        insert_if_some(
            "attendance_system",
            school
                .attendance_system
                .map(|v| bson::Bson::String(v.to_string())),
        );
        insert_if_some(
            "scholarships_available",
            school.scholarships_available.map(bson::Bson::Boolean),
        );
        insert_if_some(
            "classrooms",
            school.classrooms.map(|v| bson::Bson::Int32(v as i32)),
        );
        insert_if_some("library", school.library.map(bson::Bson::Boolean));
        insert_if_some(
            "labs",
            school
                .labs
                .map(|v| bson::to_bson(&v).unwrap())
                .map(|v| bson::Bson::Array(v.as_array().unwrap().to_vec())),
        );
        insert_if_some(
            "sports_extracurricular",
            school
                .sports_extracurricular
                .map(|v| bson::to_bson(&v).unwrap())
                .map(|v| bson::Bson::Array(v.as_array().unwrap().to_vec())),
        );
        insert_if_some(
            "student_capacity",
            school.student_capacity.map(|v| bson::Bson::Int32(v as i32)),
        );

        insert_if_some(
            "classrooms",
            school.classrooms.map(|v| bson::Bson::Int32(v as i32)),
        );
        insert_if_some(
            "online_classes",
            school.online_classes.map(bson::Bson::Boolean),
        );
        insert_if_some(
            "registration_number",
            school.registration_number.map(bson::Bson::String),
        );
        insert_if_some(
            "accreditation_body",
            school.accreditation_body.map(bson::Bson::String),
        );
        insert_if_some("school_motto", school.school_motto.map(bson::Bson::String));
        insert_if_some("logo", school.logo.map(bson::Bson::String));
        insert_if_some("is_active", school.is_active.map(bson::Bson::Boolean));

        if is_updated {
            set_doc.insert("update_at", bson::Bson::DateTime(DateTime::now()));
        }

        set_doc
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
