ALTER TABLE exams
  ADD COLUMN IF NOT EXISTS education_year_id TEXT REFERENCES education_years(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS term_id TEXT REFERENCES terms(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS description TEXT,
  ADD COLUMN IF NOT EXISTS exam_type TEXT NOT NULL DEFAULT 'Continuous',
  ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'Draft',
  ADD COLUMN IF NOT EXISTS created_by TEXT REFERENCES users(id) ON DELETE SET NULL;

UPDATE exams SET starts_at = created_at WHERE starts_at IS NULL;
UPDATE exams SET ends_at = created_at WHERE ends_at IS NULL;

CREATE INDEX IF NOT EXISTS exams_school_year_status_idx ON exams (school_id, education_year_id, status);
CREATE INDEX IF NOT EXISTS exams_school_class_start_idx ON exams (school_id, class_id, starts_at DESC);
CREATE INDEX IF NOT EXISTS exams_term_idx ON exams (term_id);
