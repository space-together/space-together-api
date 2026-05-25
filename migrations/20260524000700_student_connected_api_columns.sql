ALTER TABLE student_profiles
  ADD COLUMN IF NOT EXISTS date_of_birth_year INTEGER,
  ADD COLUMN IF NOT EXISTS date_of_birth_month INTEGER,
  ADD COLUMN IF NOT EXISTS date_of_birth_day INTEGER;

ALTER TABLE student_profiles
  DROP COLUMN IF EXISTS date_of_birth;

ALTER TABLE student_school_enrollments
  ADD COLUMN IF NOT EXISTS creator_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS deleted_by TEXT REFERENCES users(id) ON DELETE SET NULL;

CREATE TABLE IF NOT EXISTS student_enrollment_tags (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  enrollment_id TEXT NOT NULL REFERENCES student_school_enrollments(id) ON DELETE CASCADE,
  tag TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (enrollment_id, tag)
);

CREATE INDEX IF NOT EXISTS student_enrollment_tags_enrollment_idx ON student_enrollment_tags (enrollment_id, position);
