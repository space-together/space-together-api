use chrono::Utc;
use dotenvy::dotenv;
use mongodb::{
    bson::{doc, Bson, DateTime, Document},
    options::IndexOptions,
    Client, Collection, IndexModel,
};
use std::env;

type SeedResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
struct SeedItem {
    filter_field: &'static str,
    filter_value: &'static str,
    extra_null_field: Option<&'static str>,
    document: Document,
}

#[tokio::main]
async fn main() -> SeedResult<()> {
    dotenv().ok();

    let uri = env::var("MONGO_URI").expect("MONGO_URI not set in .env");
    let db_name = env::var("SEED_DB_NAME")
        .or_else(|_| env::var("MAIN_DB_NAME"))
        .unwrap_or_else(|_| "space_together".to_string());

    let client = Client::with_uri_str(uri).await?;
    let db = client.database(&db_name);

    println!("Seeding database: {db_name}");

    seed_collection(
        &db.collection::<Document>("sectors"),
        "sectors",
        &[unique_index("username"), unique_index("name")],
        &sectors(),
    )
    .await?;

    seed_collection(
        &db.collection::<Document>("main_classes"),
        "main_classes",
        &[unique_index("username"), unique_index("name")],
        &main_classes(),
    )
    .await?;

    seed_collection(
        &db.collection::<Document>("template_subjects"),
        "template_subjects",
        &[unique_index("code")],
        &template_subjects(),
    )
    .await?;

    seed_collection(
        &db.collection::<Document>("roles"),
        "roles",
        &[unique_compound_index(&[("school_id", 1), ("name", 1)])],
        &system_roles(),
    )
    .await?;

    println!("Seed completed.");
    Ok(())
}

async fn seed_collection(
    collection: &Collection<Document>,
    label: &str,
    indexes: &[IndexModel],
    items: &[SeedItem],
) -> SeedResult<()> {
    for index in indexes {
        collection.create_index(index.clone()).await?;
    }

    let mut inserted = 0;
    let mut updated = 0;

    for item in items {
        let mut filter = doc! { item.filter_field: item.filter_value };
        if let Some(field) = item.extra_null_field {
            filter.insert(field, Bson::Null);
        }

        let update = doc! {
            "$set": item.document.clone(),
            "$setOnInsert": {
                "created_at": DateTime::now()
            }
        };
        let result = collection.update_one(filter, update).upsert(true).await?;

        if result.upserted_id.is_some() {
            inserted += 1;
        } else if result.modified_count > 0 {
            updated += 1;
        }
    }

    println!(
        "{label}: {} checked, {inserted} inserted, {updated} updated",
        items.len()
    );
    Ok(())
}

fn unique_index(field: &str) -> IndexModel {
    IndexModel::builder()
        .keys(doc! { field: 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build()
}

fn unique_compound_index(fields: &[(&str, i32)]) -> IndexModel {
    let mut keys = Document::new();
    for (field, order) in fields {
        keys.insert(*field, *order);
    }

    IndexModel::builder()
        .keys(keys)
        .options(IndexOptions::builder().unique(true).build())
        .build()
}

fn timestamps(mut doc: Document) -> Document {
    doc.insert(
        "updated_at",
        DateTime::from_millis(Utc::now().timestamp_millis()),
    );
    doc.insert("disable", false);
    doc
}

fn sectors() -> Vec<SeedItem> {
    vec![
        SeedItem {
            filter_field: "username",
            filter_value: "rwanda-basic-education",
            extra_null_field: None,
            document: {
                timestamps(doc! {
                    "name": "Rwanda Basic Education",
                    "username": "rwanda-basic-education",
                    "description": "Default Rwanda primary and secondary education sector.",
                    "country": "Rwanda",
                    "type": "local",
                    "curriculum": Bson::Array(vec![Bson::Int32(1), Bson::Int32(12)]),
                    "logo": Bson::Null,
                    "logo_id": Bson::Null,
                })
            },
        },
        SeedItem {
            filter_field: "username",
            filter_value: "tvet",
            extra_null_field: None,
            document: {
                timestamps(doc! {
                    "name": "TVET",
                    "username": "tvet",
                    "description": "Technical and vocational education and training.",
                    "country": "Rwanda",
                    "type": "local",
                    "curriculum": Bson::Array(vec![Bson::Int32(10), Bson::Int32(12)]),
                    "logo": Bson::Null,
                    "logo_id": Bson::Null,
                })
            },
        },
    ]
}

fn main_classes() -> Vec<SeedItem> {
    vec![
        main_class("Primary 1", "primary-1", 1),
        main_class("Primary 2", "primary-2", 2),
        main_class("Primary 3", "primary-3", 3),
        main_class("Primary 4", "primary-4", 4),
        main_class("Primary 5", "primary-5", 5),
        main_class("Primary 6", "primary-6", 6),
        main_class("Senior 1", "senior-1", 7),
        main_class("Senior 2", "senior-2", 8),
        main_class("Senior 3", "senior-3", 9),
        main_class("Senior 4", "senior-4", 10),
        main_class("Senior 5", "senior-5", 11),
        main_class("Senior 6", "senior-6", 12),
    ]
}

fn main_class(name: &'static str, username: &'static str, level: i32) -> SeedItem {
    SeedItem {
        filter_field: "username",
        filter_value: username,
        extra_null_field: None,
        document: {
            timestamps(doc! {
                "name": name,
                "username": username,
                "level": level,
                "description": format!("Default class level {level}."),
                "trade_id": Bson::Null,
            })
        },
    }
}

fn template_subjects() -> Vec<SeedItem> {
    vec![
        subject(
            "Mathematics",
            "MATH",
            "Core mathematics subject.",
            "Mathematics",
            120,
        ),
        subject(
            "English",
            "ENG",
            "English language subject.",
            "Language",
            120,
        ),
        subject(
            "Kinyarwanda",
            "KIN",
            "Kinyarwanda language subject.",
            "Language",
            120,
        ),
        subject("Science", "SCI", "General science subject.", "Science", 120),
        subject(
            "Social Studies",
            "SST",
            "Social studies and citizenship subject.",
            "SocialScience",
            90,
        ),
        subject(
            "Computer Science",
            "CS",
            "Digital literacy and computer science subject.",
            "Technology",
            90,
        ),
    ]
}

fn subject(
    name: &'static str,
    code: &'static str,
    description: &'static str,
    category: &'static str,
    estimated_hours: i32,
) -> SeedItem {
    SeedItem {
        filter_field: "code",
        filter_value: code,
        extra_null_field: None,
        document: {
            timestamps(doc! {
                "name": name,
                "code": code,
                "description": description,
                "category": category,
                "estimated_hours": estimated_hours,
                "credits": Bson::Null,
                "prerequisites": Bson::Array(vec![]),
                "topics": Bson::Array(vec![]),
                "created_by": Bson::Null,
            })
        },
    }
}

fn system_roles() -> Vec<SeedItem> {
    vec![
        role(
            "Admin",
            "System administrator with all seeded permissions.",
            &[
                "assignment.create",
                "assignment.read.school",
                "assignment.update",
                "assignment.delete",
                "submission.grade",
                "submission.read.school",
                "role.assign",
                "role.create",
                "role.update",
                "role.delete",
                "feature.toggle",
            ],
        ),
        role(
            "Teacher",
            "Teacher role for class and assignment work.",
            &[
                "assignment.create",
                "assignment.read.class",
                "assignment.update",
                "submission.grade",
                "submission.read.class",
            ],
        ),
        role(
            "Student",
            "Student role for personal assignments and submissions.",
            &["assignment.read.own", "submission.read.own"],
        ),
        role(
            "Parent",
            "Parent role for child assignment and submission visibility.",
            &[
                "parent.read.child.assignment",
                "parent.read.child.submission",
            ],
        ),
        role(
            "School Staff",
            "School staff role for school-level assignment visibility.",
            &["assignment.read.school", "submission.read.school"],
        ),
    ]
}

fn role(
    name: &'static str,
    description: &'static str,
    permissions: &'static [&'static str],
) -> SeedItem {
    SeedItem {
        filter_field: "name",
        filter_value: name,
        extra_null_field: Some("school_id"),
        document: {
            let permissions = permissions
                .iter()
                .map(|permission| Bson::String((*permission).to_string()))
                .collect::<Vec<_>>();

            timestamps(doc! {
                "school_id": Bson::Null,
                "name": name,
                "description": description,
                "role_type": "System",
                "permissions": Bson::Array(permissions),
                "is_active": true,
            })
        },
    }
}
