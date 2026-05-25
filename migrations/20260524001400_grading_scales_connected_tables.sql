CREATE TABLE IF NOT EXISTS grading_scales (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  education_year_id TEXT REFERENCES education_years(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  grading_type TEXT NOT NULL DEFAULT 'Letter',
  is_active BOOLEAN NOT NULL DEFAULT false,
  created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS grading_scales_school_year_idx ON grading_scales (school_id, education_year_id, is_active);
CREATE UNIQUE INDEX IF NOT EXISTS grading_scales_school_year_name_unique ON grading_scales (school_id, education_year_id, lower(name)) WHERE deleted_at IS NULL;
CREATE TRIGGER grading_scales_set_updated_at BEFORE UPDATE ON grading_scales
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS grading_scale_boundaries (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  grading_scale_id TEXT NOT NULL REFERENCES grading_scales(id) ON DELETE CASCADE,
  grade TEXT NOT NULL,
  min_score DOUBLE PRECISION NOT NULL,
  max_score DOUBLE PRECISION NOT NULL,
  gpa_value DOUBLE PRECISION,
  description TEXT,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK (min_score <= max_score)
);

CREATE INDEX IF NOT EXISTS grading_scale_boundaries_scale_idx ON grading_scale_boundaries (grading_scale_id, position);
