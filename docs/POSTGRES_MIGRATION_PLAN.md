# PostgreSQL Migration Plan

## Goal

Move Space Together from MongoDB with one database per school to PostgreSQL with one connected data system, while keeping the public API responses, request bodies, routes, and school token behavior stable.

The main product goal is that one student can have a continuous education history from nursery through university. Schools still need privacy, so a school should only see the records it owns or records a student/previous school has allowed it to see.

## Current Shape

The backend currently uses:

- `MongoManager.main_db()` for global collections such as users, schools, sectors, trades, roles, join requests, and templates.
- `MongoManager.get_db(database_name)` for school-specific databases such as students, teachers, classes, scores, attendance, finance, messages, and timetables.
- `School.database_name` and `School-Token` claims to choose the school database.
- `ObjectId` values serialized as strings in API JSON.
- Mongo `Document`, indexes, and aggregation pipelines inside services/repositories.

Because Mongo details are spread through the service layer, a direct driver replacement would create a large risk of breaking APIs. The migration should happen in phases.

## Target Data Model

Use one PostgreSQL database, not one database per school.

Core global tables:

- `users`
- `schools`
- `school_memberships`
- `student_profiles`
- `student_school_enrollments`
- `student_record_permissions`
- `audit_logs`

Academic tables:

- `classes`
- `subjects`
- `education_years`
- `terms`
- `exams`
- `assessment_categories`
- `scores`
- `attendance`
- `student_term_results`
- `learning_materials`
- `assignments`

Communication and operations tables:

- `conversations`
- `messages`
- `parents`
- `guardians`
- `finance_records`
- `school_timetables`
- `class_timetables`

Important relationship:

```text
student_profiles
  -> student_school_enrollments
      -> schools
      -> classes / subjects / scores / attendance / results
```

This means the student is one person in the system, and each school contributes a verified part of that student's history.

## Privacy Model

Every school-owned record gets a `school_id`.

Schools can see:

- Their own records.
- Basic student identity for currently enrolled students.
- Previous academic history only when access is allowed.

Access is controlled by `student_record_permissions`:

- `student_id`
- `owner_school_id`
- `viewer_school_id`
- `scope`, for example `basic_profile`, `transcript`, `attendance`, `finance`, `discipline`
- `status`, for example `pending`, `allowed`, `revoked`, `expired`
- `granted_by`
- `granted_at`
- `expires_at`

For stronger database-level privacy, PostgreSQL Row Level Security can be added after the first migration:

- The API sets the active school/user context when opening a transaction.
- PostgreSQL policies prevent cross-school reads even if a query forgets a filter.

## API Compatibility Rules

The frontend should not need to change.

Keep:

- Existing routes.
- Existing request and response JSON.
- Existing `_id` field in JSON.
- Existing school token headers.
- Existing pagination shape.
- Existing role and permission checks.

To avoid breaking clients, keep legacy Mongo-style IDs as the public IDs at first:

- Store them as `TEXT` or `CHAR(24)` in PostgreSQL.
- Continue returning `_id` in API responses.
- New rows can still generate 24-character ObjectId-compatible strings until the API is ready for UUIDs.

## Implementation Phases

### Phase 1: Add PostgreSQL Beside MongoDB

Add:

- `POSTGRES_URL` environment variable.
- PostgreSQL service in Docker Compose.
- SQL migration runner.
- `PgManager` in app state.

MongoDB remains the source of truth during this phase.

### Phase 2: Create PostgreSQL Schema

Create migrations for global and school-owned data.

Important constraints:

- IDs keep the same public string shape.
- Every school-owned table has `school_id`.
- Every student history table connects through `student_id` and/or `student_enrollment_id`.
- Unique constraints include `school_id` where uniqueness is school-local.

Examples:

```sql
CREATE TABLE student_profiles (
  id CHAR(24) PRIMARY KEY,
  user_id CHAR(24) UNIQUE,
  national_id TEXT,
  name TEXT NOT NULL,
  email TEXT,
  phone TEXT,
  gender TEXT,
  date_of_birth JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE student_school_enrollments (
  id CHAR(24) PRIMARY KEY,
  student_id CHAR(24) NOT NULL REFERENCES student_profiles(id),
  school_id CHAR(24) NOT NULL REFERENCES schools(id),
  class_id CHAR(24),
  registration_number TEXT,
  admission_year INTEGER,
  status TEXT NOT NULL,
  started_at TIMESTAMPTZ,
  ended_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (school_id, registration_number)
);
```

### Phase 3: Compatibility Repository

Replace Mongo-specific access behind repository interfaces.

Do not start by rewriting every route. Keep handlers and service method names stable, then change the storage internals.

Recommended path:

1. Create a storage context that represents either global scope or school scope.
2. Replace `MongoManager.main_db()` and `MongoManager.get_db()` with methods that return that storage context.
3. Move generic CRUD from `BaseRepository` to a PostgreSQL-backed repository.
4. Convert high-risk Mongo aggregation pipelines into explicit SQL queries one feature at a time.

Short-term compatibility can use `JSONB` for fields that are still document-shaped. Long-term student history, enrollment, scores, attendance, and permissions should be relational.

### Phase 4: Data Migration

Write a migration command that:

1. Reads all schools from the current main Mongo database.
2. Creates each school in PostgreSQL.
3. Reads every school database from `School.database_name`.
4. Imports students into `student_profiles`.
5. Imports each school's student records into `student_school_enrollments`.
6. Imports classes, subjects, scores, attendance, results, finance, messages, and other records with their `school_id`.
7. Writes an `id_map` table for any ID that changes.
8. Runs counts and sample checks after each collection/table.

The most important merge rule:

```text
Same student across schools should become one student_profile,
with multiple student_school_enrollments.
```

Match students by safest available identifiers in this order:

1. Existing shared `user_id`.
2. Verified national ID, if present later.
3. Verified email or phone plus name/date of birth.
4. Manual review queue when the match is uncertain.

### Phase 5: Dual Write and Verification

For a short period:

- Keep Mongo reads active.
- Write new changes to both Mongo and PostgreSQL.
- Compare counts and important API responses.
- Fix mismatches before switching reads.

### Phase 6: Switch Reads to PostgreSQL

Switch one module at a time:

1. Auth/users/schools.
2. Students/enrollments.
3. Classes/subjects.
4. Scores/results/attendance.
5. Messaging and finance.
6. Analytics and backups.

After each module:

- Run API tests.
- Compare old Mongo response JSON with new PostgreSQL response JSON.
- Keep the same status codes and response fields.

### Phase 7: Remove Mongo Dependency

Only after PostgreSQL reads and writes are stable:

- Remove MongoDB dependency from `Cargo.toml`.
- Remove Mongo Docker service.
- Remove `database_name` as a physical database selector.
- Keep `database_name` as a deprecated API field if older clients still expect it.

## Code Areas To Change

Main backend files:

- `src/config/db.rs`
- `src/config/mongo_manager.rs`
- `src/config/state.rs`
- `src/repositories/base_repo.rs`
- `src/repositories/user_repo.rs`
- `src/services/*_service.rs`
- `src/pipeline/*_pipeline.rs`
- `src/middleware/tenant_middleware.rs`
- `src/utils/db_utils.rs`
- `src/api/school_api.rs`
- `src/utils/school_token.rs`
- `src/models/id_model.rs`

Frontend apps should not need API changes if the backend keeps JSON compatibility.

## First Code Step

The first safe code step is not to rewrite all services. It should be:

1. Add PostgreSQL config and Docker support.
2. Add migrations for `users`, `schools`, `student_profiles`, `student_school_enrollments`, and `student_record_permissions`.
3. Add a migration/import command that can read Mongo and write PostgreSQL.
4. Test import counts.
5. Then start replacing the repository internals module by module.

This keeps the product online while moving from separate school databases to one connected education history.
