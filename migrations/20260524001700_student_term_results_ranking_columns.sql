ALTER TABLE student_term_results
  ADD COLUMN IF NOT EXISTS education_year_id TEXT REFERENCES education_years(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS exam_id TEXT REFERENCES exams(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS term_id TEXT,
  ADD COLUMN IF NOT EXISTS total_max_score DOUBLE PRECISION NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS average_percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS gpa DOUBLE PRECISION NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS total_credits INTEGER,
  ADD COLUMN IF NOT EXISTS grade TEXT,
  ADD COLUMN IF NOT EXISTS rank_in_class INTEGER,
  ADD COLUMN IF NOT EXISTS total_students INTEGER,
  ADD COLUMN IF NOT EXISTS calculated_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS is_finalized BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE student_term_results
  ALTER COLUMN total_score TYPE DOUBLE PRECISION USING total_score::DOUBLE PRECISION,
  ALTER COLUMN average_score TYPE DOUBLE PRECISION USING average_score::DOUBLE PRECISION;

UPDATE student_term_results
SET term_id = COALESCE(term_id, term),
    average_percentage = COALESCE(NULLIF(average_percentage, 0), average_score, 0),
    gpa = COALESCE(NULLIF(gpa, 0), average_score, 0),
    rank_in_class = COALESCE(rank_in_class, rank),
    calculated_at = COALESCE(calculated_at, updated_at, created_at)
WHERE term_id IS NULL
   OR average_percentage = 0
   OR gpa = 0
   OR rank_in_class IS NULL
   OR calculated_at IS NULL;

CREATE INDEX IF NOT EXISTS student_term_results_class_exam_rank_idx
  ON student_term_results (class_id, exam_id, rank_in_class);
CREATE INDEX IF NOT EXISTS student_term_results_class_exam_gpa_idx
  ON student_term_results (class_id, exam_id, gpa DESC, average_percentage DESC);
CREATE INDEX IF NOT EXISTS student_term_results_education_year_idx
  ON student_term_results (education_year_id);
