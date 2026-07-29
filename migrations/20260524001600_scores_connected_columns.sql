ALTER TABLE scores
  ADD COLUMN IF NOT EXISTS education_year_id TEXT REFERENCES education_years(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS remarks TEXT,
  ADD COLUMN IF NOT EXISTS entered_by TEXT REFERENCES users(id) ON DELETE SET NULL;

ALTER TABLE scores
  ALTER COLUMN score TYPE DOUBLE PRECISION USING score::DOUBLE PRECISION,
  ALTER COLUMN max_score TYPE DOUBLE PRECISION USING max_score::DOUBLE PRECISION;

UPDATE scores
SET entered_by = COALESCE(entered_by, recorded_by),
    percentage = CASE
      WHEN max_score > 0 THEN (score / max_score) * 100
      ELSE 0
    END
WHERE entered_by IS NULL OR percentage = 0;

CREATE UNIQUE INDEX IF NOT EXISTS scores_student_subject_exam_category_unique
  ON scores (student_id, class_subject_id, exam_id, assessment_category_id)
  WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS scores_school_exam_idx ON scores (school_id, exam_id);
CREATE INDEX IF NOT EXISTS scores_education_year_idx ON scores (education_year_id);
CREATE INDEX IF NOT EXISTS scores_entered_by_idx ON scores (entered_by);

ALTER TABLE score_audit_logs
  ADD COLUMN IF NOT EXISTS change_reason TEXT,
  ADD COLUMN IF NOT EXISTS changed_at TIMESTAMPTZ;

ALTER TABLE score_audit_logs
  ALTER COLUMN old_score TYPE DOUBLE PRECISION USING old_score::DOUBLE PRECISION,
  ALTER COLUMN new_score TYPE DOUBLE PRECISION USING new_score::DOUBLE PRECISION;

UPDATE score_audit_logs
SET change_reason = COALESCE(change_reason, reason),
    changed_at = COALESCE(changed_at, created_at)
WHERE change_reason IS NULL OR changed_at IS NULL;

CREATE INDEX IF NOT EXISTS score_audit_logs_changed_at_idx ON score_audit_logs (score_id, changed_at DESC);
