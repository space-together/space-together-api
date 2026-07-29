ALTER TABLE education_years
  ADD COLUMN IF NOT EXISTS created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS deleted_by TEXT REFERENCES users(id) ON DELETE SET NULL;

CREATE UNIQUE INDEX IF NOT EXISTS education_years_school_curriculum_name_unique
ON education_years (school_id, curriculum_id, lower(name))
WHERE deleted_at IS NULL;

ALTER TABLE terms
  ADD COLUMN IF NOT EXISTS term_order INTEGER NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS terms_year_order_idx ON terms (education_year_id, term_order);
