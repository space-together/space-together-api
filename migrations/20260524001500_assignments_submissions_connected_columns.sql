ALTER TABLE assignments
  ADD COLUMN IF NOT EXISTS subject_id TEXT REFERENCES class_subjects(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS teacher_id TEXT REFERENCES teachers(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS instructions TEXT,
  ADD COLUMN IF NOT EXISTS due_date TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS max_score DOUBLE PRECISION NOT NULL DEFAULT 100,
  ADD COLUMN IF NOT EXISTS allow_late_submission BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS attachment_url TEXT,
  ADD COLUMN IF NOT EXISTS attachment_id TEXT,
  ADD COLUMN IF NOT EXISTS auto_grade_enabled BOOLEAN NOT NULL DEFAULT false;

UPDATE assignments
SET subject_id = COALESCE(subject_id, class_subject_id),
    due_date = COALESCE(due_date, due_at)
WHERE subject_id IS NULL OR due_date IS NULL;

CREATE INDEX IF NOT EXISTS assignments_subject_id_idx ON assignments (subject_id);
CREATE INDEX IF NOT EXISTS assignments_teacher_id_idx ON assignments (teacher_id);
CREATE INDEX IF NOT EXISTS assignments_school_status_due_idx ON assignments (school_id, status, due_date);

ALTER TABLE submissions
  ADD COLUMN IF NOT EXISTS file_url TEXT,
  ADD COLUMN IF NOT EXISTS file_id TEXT,
  ADD COLUMN IF NOT EXISTS comment TEXT,
  ADD COLUMN IF NOT EXISTS is_late BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS feedback_file_url TEXT,
  ADD COLUMN IF NOT EXISTS feedback_file_id TEXT,
  ADD COLUMN IF NOT EXISTS graded_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS graded_by TEXT REFERENCES teachers(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS auto_grade_score DOUBLE PRECISION,
  ADD COLUMN IF NOT EXISTS ai_feedback TEXT;

ALTER TABLE submissions
  ALTER COLUMN score TYPE DOUBLE PRECISION USING score::DOUBLE PRECISION;

CREATE INDEX IF NOT EXISTS submissions_assignment_student_idx ON submissions (assignment_id, student_id);
CREATE INDEX IF NOT EXISTS submissions_graded_by_idx ON submissions (graded_by);
CREATE INDEX IF NOT EXISTS submissions_status_submitted_idx ON submissions (status, submitted_at DESC);

CREATE TABLE IF NOT EXISTS school_feature_flags (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  feature_name TEXT NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT true,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (school_id, feature_name)
);

CREATE INDEX IF NOT EXISTS school_feature_flags_school_idx ON school_feature_flags (school_id);
CREATE TRIGGER school_feature_flags_set_updated_at BEFORE UPDATE ON school_feature_flags
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
