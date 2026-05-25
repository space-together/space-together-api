ALTER TABLE assessment_categories
  ADD COLUMN IF NOT EXISTS education_year_id TEXT REFERENCES education_years(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS code TEXT,
  ADD COLUMN IF NOT EXISTS weight_percentage NUMERIC,
  ADD COLUMN IF NOT EXISTS description TEXT,
  ADD COLUMN IF NOT EXISTS created_by TEXT REFERENCES users(id) ON DELETE SET NULL;

UPDATE assessment_categories
SET weight_percentage = weight
WHERE weight_percentage IS NULL AND weight IS NOT NULL;

CREATE INDEX IF NOT EXISTS assessment_categories_year_idx ON assessment_categories (education_year_id);
CREATE INDEX IF NOT EXISTS assessment_categories_subject_year_idx ON assessment_categories (class_subject_id, education_year_id);
CREATE UNIQUE INDEX IF NOT EXISTS assessment_categories_school_subject_year_code_unique
ON assessment_categories (school_id, class_subject_id, education_year_id, lower(code))
WHERE code IS NOT NULL AND deleted_at IS NULL;
