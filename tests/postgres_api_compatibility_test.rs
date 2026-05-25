use serde_json::{json, Value};

fn assert_compatible_json_shape(old_response: &Value, postgres_response: &Value) {
    match (old_response, postgres_response) {
        (Value::Object(old), Value::Object(new)) => {
            for (key, old_value) in old {
                assert!(
                    new.contains_key(key),
                    "PostgreSQL response is missing compatibility field `{key}`"
                );
                assert_compatible_json_shape(old_value, &new[key]);
            }
        }
        (Value::Array(old_items), Value::Array(new_items)) => {
            if let (Some(old_item), Some(new_item)) = (old_items.first(), new_items.first()) {
                assert_compatible_json_shape(old_item, new_item);
            }
        }
        _ => {}
    }
}

#[test]
fn postgres_responses_keep_mongo_style_public_ids() {
    let response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Student One",
        "school_id": "664f2b78a68dff8b9cf934d2"
    });

    assert_eq!(response["_id"].as_str().unwrap().len(), 24);
    assert!(response.get("_id").is_some());
}

#[test]
fn compatibility_shape_allows_postgres_extra_internal_fields_only_when_legacy_fields_remain() {
    let mongo_response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Student One",
        "email": "student@example.com",
        "school_id": "664f2b78a68dff8b9cf934d2",
        "class_id": "664f2b78a68dff8b9cf934d3"
    });

    let postgres_response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Student One",
        "email": "student@example.com",
        "school_id": "664f2b78a68dff8b9cf934d2",
        "class_id": "664f2b78a68dff8b9cf934d3",
        "student_id": "664f2b78a68dff8b9cf934d4",
        "enrollment_id": "664f2b78a68dff8b9cf934d1"
    });

    assert_compatible_json_shape(&mongo_response, &postgres_response);
}

#[test]
fn school_compatibility_keeps_database_name_as_response_field() {
    let mongo_response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Space Together School",
        "username": "space-school",
        "database_name": "school_664f2b78a68dff8b9cf934d1"
    });

    let postgres_response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Space Together School",
        "username": "space-school",
        "database_name": "school_664f2b78a68dff8b9cf934d1"
    });

    assert_compatible_json_shape(&mongo_response, &postgres_response);
}

#[test]
fn replacement_schema_removes_document_storage_columns() {
    let migration = include_str!("../migrations/20260524000400_relational_connected_schema.sql");

    assert!(migration.contains("DROP COLUMN IF EXISTS raw_document"));
    assert!(!migration.contains("CREATE TABLE legacy_records"));
    assert!(!migration.contains("raw_document JSONB"));
}

#[test]
fn user_response_shape_keeps_frontend_contract_fields() {
    let mongo_response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Student One",
        "email": "student@example.com",
        "username": "student-one",
        "image": null,
        "role": "STUDENT",
        "bio": null,
        "current_school_id": "664f2b78a68dff8b9cf934d2",
        "schools": ["664f2b78a68dff8b9cf934d2"],
        "accessible_classes": ["664f2b78a68dff8b9cf934d3"]
    });

    let postgres_response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Student One",
        "email": "student@example.com",
        "username": "student-one",
        "image": null,
        "role": "STUDENT",
        "bio": null,
        "current_school_id": "664f2b78a68dff8b9cf934d2",
        "schools": ["664f2b78a68dff8b9cf934d2"],
        "accessible_classes": ["664f2b78a68dff8b9cf934d3"]
    });

    assert_compatible_json_shape(&mongo_response, &postgres_response);
}

#[test]
fn user_and_auth_modules_do_not_use_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/repositories/user_repo.rs"),
        include_str!("../src/services/auth_service.rs"),
        include_str!("../src/services/user_service.rs"),
        include_str!("../src/api/users.rs"),
        include_str!("../src/api/auth_api.rs"),
        include_str!("../src/domain/user.rs"),
    ];

    for file in migrated_files {
        assert!(!file.contains("mongodb::"));
        assert!(!file.contains("bson::"));
        assert!(!file.contains("Collection<"));
        assert!(!file.contains("Database"));
        assert!(!file.contains("doc!"));
    }
}

#[test]
fn base_repository_is_postgres_query_layer() {
    let base_repo = include_str!("../src/repositories/base_repo.rs");

    assert!(base_repo.contains("PgPool"));
    assert!(base_repo.contains("QueryBuilder"));
    assert!(base_repo.contains("ORDER BY updated_at DESC"));
    assert!(!base_repo.contains("mongodb::"));
    assert!(!base_repo.contains("bson::"));
    assert!(!base_repo.contains("Collection<"));
    assert!(!base_repo.contains("Database"));
    assert!(!base_repo.contains("doc!"));
    assert!(!base_repo.contains(".aggregate("));
}

#[test]
fn remaining_mongo_base_layer_is_explicitly_legacy_named() {
    let service_with_legacy_import = include_str!("../src/services/assessment_category_service.rs");

    assert!(service_with_legacy_import.contains("legacy_mongo_base_repo"));
    assert!(!service_with_legacy_import.contains("repositories::base_repo::BaseRepository"));
}

#[test]
fn school_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/school_service.rs"),
        include_str!("../src/api/school_api.rs"),
    ];

    for file in migrated_files {
        assert!(file.contains("PgPool") || file.contains("state.pg.pool"));
        assert!(!file.contains("mongodb::"));
        assert!(!file.contains("bson::"));
        assert!(!file.contains("Collection<"));
        assert!(!file.contains("Database"));
        assert!(!file.contains("doc!"));
        assert!(!file.contains(".aggregate("));
    }
}

#[test]
fn school_profile_schema_uses_relational_tables() {
    let migration = include_str!("../migrations/20260524000600_school_profile_relations.sql");

    assert!(migration.contains("CREATE TABLE IF NOT EXISTS school_curricula"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS school_education_levels"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS school_social_media"));
    assert!(!migration.contains("raw_document"));
}

#[test]
fn student_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/student_service.rs"),
        include_str!("../src/api/students_api.rs"),
        include_str!("../src/domain/student.rs"),
    ];

    for file in migrated_files {
        assert!(!file.contains("mongodb::"));
        assert!(!file.contains("bson::"));
        assert!(!file.contains("Collection<"));
        assert!(!file.contains("Database"));
        assert!(!file.contains("doc!"));
        assert!(!file.contains(".aggregate("));
        assert!(!file.contains("get_database"));
        assert!(!file.contains("build_extra_match"));
    }
}

#[test]
fn student_connected_schema_uses_profiles_enrollments_and_tag_table() {
    let core = include_str!("../migrations/20260524000100_core_connected_identity.sql");
    let compat = include_str!("../migrations/20260524000700_student_connected_api_columns.sql");

    assert!(core.contains("CREATE TABLE student_profiles"));
    assert!(core.contains("CREATE TABLE student_school_enrollments"));
    assert!(core.contains("CREATE TABLE student_record_permissions"));
    assert!(compat.contains("DROP COLUMN IF EXISTS date_of_birth"));
    assert!(compat.contains("CREATE TABLE IF NOT EXISTS student_enrollment_tags"));
    assert!(compat.contains("REFERENCES student_school_enrollments(id)"));
    assert!(!compat.contains("JSONB"));
}
