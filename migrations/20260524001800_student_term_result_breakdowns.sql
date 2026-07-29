CREATE TABLE IF NOT EXISTS student_term_subject_results (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  result_id TEXT NOT NULL REFERENCES student_term_results(id) ON DELETE CASCADE,
  class_subject_id TEXT REFERENCES class_subjects(id) ON DELETE SET NULL,
  subject_name TEXT NOT NULL,
  weighted_score DOUBLE PRECISION NOT NULL DEFAULT 0,
  percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
  grade TEXT NOT NULL DEFAULT '',
  credits INTEGER,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS student_term_category_scores (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  subject_result_id TEXT NOT NULL REFERENCES student_term_subject_results(id) ON DELETE CASCADE,
  assessment_category_id TEXT REFERENCES assessment_categories(id) ON DELETE SET NULL,
  category_name TEXT NOT NULL,
  score DOUBLE PRECISION NOT NULL DEFAULT 0,
  max_score DOUBLE PRECISION NOT NULL DEFAULT 0,
  weight_percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE student_term_results
  DROP CONSTRAINT IF EXISTS student_term_results_school_id_student_id_term_academic_year_key;

CREATE UNIQUE INDEX IF NOT EXISTS student_term_results_student_exam_unique
  ON student_term_results (student_id, exam_id)
  WHERE exam_id IS NOT NULL AND deleted_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS student_term_results_school_student_term_year_unique
  ON student_term_results (school_id, student_id, term, academic_year)
  WHERE exam_id IS NULL AND deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS student_term_subject_results_result_idx
  ON student_term_subject_results (result_id, position);
CREATE INDEX IF NOT EXISTS student_term_subject_results_subject_idx
  ON student_term_subject_results (class_subject_id);
CREATE INDEX IF NOT EXISTS student_term_category_scores_subject_result_idx
  ON student_term_category_scores (subject_result_id, position);
CREATE INDEX IF NOT EXISTS student_term_category_scores_category_idx
  ON student_term_category_scores (assessment_category_id);
