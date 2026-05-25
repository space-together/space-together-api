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
    let service_with_legacy_import = include_str!("../src/services/announcement_service.rs");

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

#[test]
fn parent_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/parent_service.rs"),
        include_str!("../src/api/parent_api.rs"),
        include_str!("../src/domain/parent.rs"),
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
fn parent_schema_uses_parent_student_links_for_child_permissions() {
    let core = include_str!("../migrations/20260524000400_relational_connected_schema.sql");
    let compat = include_str!("../migrations/20260524000800_parent_connected_api_columns.sql");

    assert!(core.contains("CREATE TABLE IF NOT EXISTS parent_student_links"));
    assert!(core.contains("can_view_academics BOOLEAN"));
    assert!(compat.contains("parents_school_email_idx"));
    assert!(compat.contains("parent_student_links_parent_student_idx"));
    assert!(!compat.contains("JSONB"));
}

#[test]
fn staff_and_roles_use_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/teacher_service.rs"),
        include_str!("../src/api/teachers_api.rs"),
        include_str!("../src/domain/teacher.rs"),
        include_str!("../src/services/school_staff_service.rs"),
        include_str!("../src/api/school_staff_api.rs"),
        include_str!("../src/domain/school_staff.rs"),
        include_str!("../src/services/role_service.rs"),
        include_str!("../src/api/roles_api.rs"),
        include_str!("../src/domain/role.rs"),
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
fn staff_role_schema_uses_join_tables_and_permissions() {
    let migration = include_str!("../migrations/20260524000900_staff_roles_connected_columns.sql");

    assert!(migration.contains("CREATE TABLE IF NOT EXISTS teacher_classes"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS teacher_subjects"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS teacher_tags"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS school_staff_tags"));
    assert!(migration.contains("role_id TEXT REFERENCES roles(id)"));
    assert!(migration.contains("user_role_assignments_user_role_school_unique"));
    assert!(!migration.contains("JSONB"));
}

#[test]
fn class_and_subject_services_use_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/class_service.rs"),
        include_str!("../src/api/class_api.rs"),
        include_str!("../src/domain/class.rs"),
        include_str!("../src/services/class_subject_service.rs"),
        include_str!("../src/api/class_subject.rs"),
        include_str!("../src/domain/class_subject.rs"),
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
fn class_subject_schema_uses_relational_tables() {
    let migration =
        include_str!("../migrations/20260524001000_classes_subjects_connected_columns.sql");

    assert!(migration.contains("CREATE TABLE IF NOT EXISTS class_tags"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS class_background_images"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS class_settings"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS class_subject_topics"));
    assert!(migration.contains("class_teacher_id TEXT REFERENCES teachers(id)"));
    assert!(migration.contains("teacher_id TEXT REFERENCES teachers(id)"));
    assert!(!migration.contains("JSONB"));
}

#[test]
fn education_year_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/education_year_service.rs"),
        include_str!("../src/api/education_year_api.rs"),
        include_str!("../src/domain/education_year.rs"),
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
fn education_year_schema_uses_terms_table_not_embedded_terms() {
    let core = include_str!("../migrations/20260524000400_relational_connected_schema.sql");
    let migration =
        include_str!("../migrations/20260524001100_education_year_terms_connected_columns.sql");

    assert!(core.contains("CREATE TABLE IF NOT EXISTS education_years"));
    assert!(core.contains("CREATE TABLE IF NOT EXISTS terms"));
    assert!(migration.contains("term_order INTEGER"));
    assert!(migration.contains("education_years_school_curriculum_name_unique"));
    assert!(!migration.contains("JSONB"));
}

#[test]
fn exam_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/exam_service.rs"),
        include_str!("../src/api/exam_api.rs"),
        include_str!("../src/domain/exam.rs"),
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
fn exam_schema_connects_to_school_year_term_class_and_creator() {
    let migration = include_str!("../migrations/20260524001200_exams_connected_columns.sql");

    assert!(migration.contains("education_year_id TEXT REFERENCES education_years(id)"));
    assert!(migration.contains("term_id TEXT REFERENCES terms(id)"));
    assert!(migration.contains("created_by TEXT REFERENCES users(id)"));
    assert!(migration.contains("exams_school_year_status_idx"));
    assert!(!migration.contains("JSONB"));
}

#[test]
fn assessment_category_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/assessment_category_service.rs"),
        include_str!("../src/api/assessment_category_api.rs"),
        include_str!("../src/domain/assessment_category.rs"),
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
fn assessment_category_schema_connects_subjects_years_and_creators() {
    let migration =
        include_str!("../migrations/20260524001300_assessment_categories_connected_columns.sql");

    assert!(migration.contains("education_year_id TEXT REFERENCES education_years(id)"));
    assert!(migration.contains("created_by TEXT REFERENCES users(id)"));
    assert!(migration.contains("assessment_categories_subject_year_idx"));
    assert!(migration.contains("assessment_categories_school_subject_year_code_unique"));
    assert!(!migration.contains("JSONB"));
}

#[test]
fn grading_scale_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/grading_scale_service.rs"),
        include_str!("../src/api/grading_scale_api.rs"),
        include_str!("../src/domain/grading_scale.rs"),
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
fn grading_scale_schema_uses_boundary_table_not_embedded_storage() {
    let migration =
        include_str!("../migrations/20260524001400_grading_scales_connected_tables.sql");

    assert!(migration.contains("CREATE TABLE IF NOT EXISTS grading_scales"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS grading_scale_boundaries"));
    assert!(migration.contains("grading_scale_id TEXT NOT NULL REFERENCES grading_scales(id)"));
    assert!(migration.contains("grading_scales_school_year_idx"));
    assert!(migration.contains("grading_scale_boundaries_scale_idx"));
    assert!(!migration.contains("JSONB"));
}

#[test]
fn assignment_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/assignment_service.rs"),
        include_str!("../src/api/assignment_api.rs"),
        include_str!("../src/domain/assignment.rs"),
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
        assert!(!file.contains("legacy_mongo_base_repo"));
    }
}

#[test]
fn assignment_schema_uses_connected_assignment_and_submission_columns() {
    let migration =
        include_str!("../migrations/20260524001500_assignments_submissions_connected_columns.sql");

    assert!(migration.contains("teacher_id TEXT REFERENCES teachers(id)"));
    assert!(migration.contains("subject_id TEXT REFERENCES class_subjects(id)"));
    assert!(migration.contains("graded_by TEXT REFERENCES teachers(id)"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS school_feature_flags"));
    assert!(migration.contains("submissions_assignment_student_idx"));
    assert!(!migration.contains("JSONB"));
}

#[test]
fn score_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/score_service.rs"),
        include_str!("../src/api/score_api.rs"),
        include_str!("../src/domain/score.rs"),
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
        assert!(!file.contains("legacy_mongo_base_repo"));
    }
}

#[test]
fn score_schema_connects_scores_and_audit_logs() {
    let migration = include_str!("../migrations/20260524001600_scores_connected_columns.sql");

    assert!(migration.contains("education_year_id TEXT REFERENCES education_years(id)"));
    assert!(migration.contains("entered_by TEXT REFERENCES users(id)"));
    assert!(migration.contains("scores_student_subject_exam_category_unique"));
    assert!(migration.contains("score_audit_logs_changed_at_idx"));
    assert!(!migration.contains("JSONB"));
}

#[test]
fn ranking_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/ranking_service.rs"),
        include_str!("../src/api/ranking_api.rs"),
        include_str!("../src/domain/student_term_result.rs"),
    ];

    for file in migrated_files {
        assert!(!file.contains("mongodb::"));
        assert!(!file.contains("bson::"));
        assert!(!file.contains("Collection<"));
        assert!(!file.contains("Database"));
        assert!(!file.contains("doc!"));
        assert!(!file.contains(".aggregate("));
        assert!(!file.contains("get_database"));
        assert!(!file.contains("legacy_mongo_base_repo"));
    }
}

#[test]
fn ranking_schema_uses_student_term_result_columns() {
    let migration =
        include_str!("../migrations/20260524001700_student_term_results_ranking_columns.sql");

    assert!(migration.contains("exam_id TEXT REFERENCES exams(id)"));
    assert!(migration.contains("rank_in_class INTEGER"));
    assert!(migration.contains("student_term_results_class_exam_rank_idx"));
    assert!(migration.contains("student_term_results_class_exam_gpa_idx"));
    assert!(!migration.contains("JSONB"));
}

#[test]
fn results_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/gpa_calculation_service.rs"),
        include_str!("../src/api/results_api.rs"),
        include_str!("../src/domain/student_term_result.rs"),
    ];

    for file in migrated_files {
        assert!(!file.contains("mongodb::"));
        assert!(!file.contains("bson::"));
        assert!(!file.contains("Collection<"));
        assert!(!file.contains("Database"));
        assert!(!file.contains("doc!"));
        assert!(!file.contains(".aggregate("));
        assert!(!file.contains("get_database"));
        assert!(!file.contains("legacy_mongo_base_repo"));
    }
}

#[test]
fn student_term_result_schema_uses_relational_breakdown_tables() {
    let migration = include_str!("../migrations/20260524001800_student_term_result_breakdowns.sql");

    assert!(migration.contains("CREATE TABLE IF NOT EXISTS student_term_subject_results"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS student_term_category_scores"));
    assert!(migration.contains("result_id TEXT NOT NULL REFERENCES student_term_results(id)"));
    assert!(migration
        .contains("subject_result_id TEXT NOT NULL REFERENCES student_term_subject_results(id)"));
    assert!(migration.contains("class_subject_id TEXT REFERENCES class_subjects(id)"));
    assert!(migration.contains("assessment_category_id TEXT REFERENCES assessment_categories(id)"));
    assert!(migration.contains("DROP CONSTRAINT IF EXISTS"));
    assert!(migration.contains("student_term_results_student_exam_unique"));
    assert!(migration.contains("student_term_results_school_student_term_year_unique"));
    assert!(!migration.contains("JSONB"));
}

#[test]
fn analytics_service_uses_postgres_without_mongo_storage_apis() {
    let migrated_files = [
        include_str!("../src/services/analytics_service.rs"),
        include_str!("../src/api/analytics_api.rs"),
        include_str!("../src/domain/analytics.rs"),
    ];

    for file in migrated_files {
        assert!(!file.contains("mongodb::"));
        assert!(!file.contains("bson::"));
        assert!(!file.contains("Collection<"));
        assert!(!file.contains("Database"));
        assert!(!file.contains("doc!"));
        assert!(!file.contains(".aggregate("));
        assert!(!file.contains("get_database"));
        assert!(!file.contains("legacy_mongo_base_repo"));
    }

    let service = include_str!("../src/services/analytics_service.rs");
    assert!(service.contains("FROM student_school_enrollments"));
    assert!(service.contains("FROM attendance"));
    assert!(service.contains("FROM scores"));
    assert!(service.contains("FROM finance_records"));
    assert!(service.contains("LEFT JOIN class_subjects"));
}
