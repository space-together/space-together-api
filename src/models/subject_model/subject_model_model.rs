use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SubjectType {
    General,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub class_room_id: Option<ObjectId>,
    pub class_id: Option<ObjectId>,
    pub code: String,
    pub sector_id: Option<ObjectId>,
    pub trade_id: Option<ObjectId>,
    pub subject_type: Option<SubjectType>,
    pub curriculum: Option<String>,
    pub copyright: Option<String>,
    pub learning_hours: Option<i32>,
    pub issue_date: Option<DateTime>,
    pub purpose: Option<String>,
    pub symbol: Option<String>,
    pub knowledge: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub attitude: Option<Vec<String>>,
    pub resource: Option<Vec<SubjectResource>>,
    pub competence: Option<Vec<SubjectCompetence>>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SubjectResourceType {
    Equipment,
    Material,
    Tools,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectResource {
    pub category: SubjectResourceType,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectCompetence {
    pub description: Option<String>,
    pub label: String,
    pub performance_criteria: Option<Vec<SubjectCompetence>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectModelGet {
    pub id: String,
    pub name: String,
    pub class_room_id: Option<String>,
    pub class_id: Option<String>,
    pub code: String,
    pub sector_id: Option<String>,
    pub trade_id: Option<String>,
    pub subject_type: Option<SubjectType>,
    pub curriculum: Option<String>,
    pub copyright: Option<String>,
    pub learning_hours: Option<i32>,
    pub issue_date: Option<String>,
    pub purpose: Option<String>,
    pub symbol: Option<String>,
    pub knowledge: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub attitude: Option<Vec<String>>,
    pub resource: Option<Vec<SubjectResource>>,
    pub competence: Option<Vec<SubjectCompetence>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectModelNew {
    pub name: String,
    pub class_room_id: Option<String>,
    pub class_id: Option<String>,
    pub code: String,
    pub sector_id: Option<String>,
    pub trade_id: Option<String>,
    pub subject_type: Option<SubjectType>,
    pub curriculum: Option<String>,
    pub copyright: Option<String>,
    pub learning_hours: Option<i32>,
    pub issue_date: Option<String>,
    pub purpose: Option<String>,
    pub symbol: Option<String>,
    pub knowledge: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub attitude: Option<Vec<String>>,
    pub resource: Option<Vec<SubjectResource>>,
    pub competence: Option<Vec<SubjectCompetence>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectModelPut {
    pub name: Option<String>,
    pub class_room_id: Option<String>,
    pub class_id: Option<String>,
    pub code: Option<String>,
    pub sector_id: Option<String>,
    pub trade_id: Option<String>,
    pub subject_type: Option<SubjectType>,
    pub curriculum: Option<String>,
    pub copyright: Option<String>,
    pub learning_hours: Option<i32>,
    pub issue_date: Option<String>,
    pub purpose: Option<String>,
    pub symbol: Option<String>,
    pub knowledge: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub attitude: Option<Vec<String>>,
    pub resource: Option<Vec<SubjectResource>>,
    pub competence: Option<Vec<SubjectCompetence>>,
}

impl SubjectModel {
    pub fn new(subject: SubjectModelNew) -> Self {
        SubjectModel {
            id: None,
            name: subject.name,
            class_room_id: subject
                .class_room_id
                .map(|id| ObjectId::from_str(&id).unwrap()),
            class_id: subject.class_id.map(|id| ObjectId::from_str(&id).unwrap()),
            code: subject.code,
            sector_id: subject.sector_id.map(|id| ObjectId::from_str(&id).unwrap()),
            trade_id: subject.trade_id.map(|id| ObjectId::from_str(&id).unwrap()),
            subject_type: subject.subject_type,
            curriculum: subject.curriculum,
            copyright: subject.copyright,
            learning_hours: subject.learning_hours,
            issue_date: subject
                .issue_date
                .map(|date| DateTime::parse_rfc3339_str(&date).unwrap()),
            purpose: subject.purpose,
            symbol: subject.symbol,
            knowledge: subject.knowledge,
            skills: subject.skills,
            attitude: subject.attitude,
            resource: subject.resource,
            competence: subject.competence,
            created_at: Some(DateTime::now()),
            updated_at: None,
        }
    }

    pub fn format(subject: Self) -> SubjectModelGet {
        SubjectModelGet {
            id: subject.id.map_or("".to_string(), |id| id.to_string()),
            name: subject.name,
            class_room_id: subject.class_room_id.map(|id| id.to_string()),
            class_id: subject.class_id.map(|id| id.to_string()),
            code: subject.code,
            sector_id: subject.sector_id.map(|id| id.to_string()),
            trade_id: subject.trade_id.map(|id| id.to_string()),
            subject_type: subject.subject_type,
            curriculum: subject.curriculum,
            copyright: subject.copyright,
            learning_hours: subject.learning_hours,
            issue_date: subject
                .issue_date
                .map(|date| date.try_to_rfc3339_string().unwrap_or("".to_string())),
            purpose: subject.purpose,
            symbol: subject.symbol,
            knowledge: subject.knowledge,
            skills: subject.skills,
            attitude: subject.attitude,
            resource: subject.resource,
            competence: subject.competence,
            created_at: subject
                .created_at
                .map(|date| date.try_to_rfc3339_string().unwrap_or("".to_string())),
            updated_at: subject
                .updated_at
                .map(|date| date.try_to_rfc3339_string().unwrap_or("".to_string())),
        }
    }

    pub fn put(subject: SubjectModelPut) -> Document {
        let mut doc = Document::new();
        let mut is_update = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
                is_update = true;
            }
        };

        insert_if_some(
            "class_room_id",
            subject
                .class_room_id
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        insert_if_some(
            "class_id",
            subject
                .class_id
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        insert_if_some(
            "sector_id",
            subject
                .sector_id
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        insert_if_some(
            "trade_id",
            subject
                .trade_id
                .map(|id| bson::Bson::ObjectId(ObjectId::from_str(&id).unwrap())),
        );

        insert_if_some("name", subject.name.map(bson::Bson::String));
        insert_if_some("code", subject.code.map(bson::Bson::String));
        insert_if_some(
            "subject_type",
            subject.subject_type.map(|st| bson::to_bson(&st).unwrap()),
        );
        insert_if_some("curriculum", subject.curriculum.map(bson::Bson::String));
        insert_if_some("copyright", subject.copyright.map(bson::Bson::String));
        insert_if_some(
            "learning_hours",
            subject.learning_hours.map(bson::Bson::Int32),
        );
        insert_if_some(
            "issue_date",
            subject
                .issue_date
                .map(|date| bson::Bson::DateTime(DateTime::parse_rfc3339_str(&date).unwrap())),
        );
        insert_if_some("purpose", subject.purpose.map(bson::Bson::String));
        insert_if_some("symbol", subject.symbol.map(bson::Bson::String));
        insert_if_some(
            "knowledge",
            subject
                .knowledge
                .map(|k| bson::Bson::Array(k.into_iter().map(bson::Bson::String).collect())),
        );
        insert_if_some(
            "skills",
            subject
                .skills
                .map(|s| bson::Bson::Array(s.into_iter().map(bson::Bson::String).collect())),
        );
        insert_if_some(
            "attitude",
            subject
                .attitude
                .map(|a| bson::Bson::Array(a.into_iter().map(bson::Bson::String).collect())),
        );
        insert_if_some(
            "resource",
            subject.resource.map(|r| {
                bson::Bson::Array(
                    r.into_iter()
                        .map(|res| bson::to_bson(&res).unwrap())
                        .collect(),
                )
            }),
        );
        insert_if_some(
            "competence",
            subject.competence.map(|c| {
                bson::Bson::Array(
                    c.into_iter()
                        .map(|comp| bson::to_bson(&comp).unwrap())
                        .collect(),
                )
            }),
        );

        if is_update {
            doc.insert("updated_at", bson::Bson::DateTime(DateTime::now()));
        }

        doc
    }
}
