ALTER TABLE learning_materials
  ADD COLUMN IF NOT EXISTS subject_id TEXT REFERENCES class_subjects(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS file_url TEXT,
  ADD COLUMN IF NOT EXISTS file_public_id TEXT,
  ADD COLUMN IF NOT EXISTS video_url TEXT,
  ADD COLUMN IF NOT EXISTS is_published BOOLEAN NOT NULL DEFAULT false;

UPDATE learning_materials
SET subject_id = COALESCE(subject_id, class_subject_id),
    file_url = COALESCE(file_url, url)
WHERE subject_id IS NULL OR file_url IS NULL;

CREATE INDEX IF NOT EXISTS learning_materials_subject_id_idx
  ON learning_materials (subject_id);
CREATE INDEX IF NOT EXISTS learning_materials_class_subject_school_idx
  ON learning_materials (school_id, class_id, subject_id, created_at DESC);
CREATE INDEX IF NOT EXISTS learning_materials_uploaded_by_idx
  ON learning_materials (uploaded_by, created_at DESC);
CREATE INDEX IF NOT EXISTS learning_materials_published_idx
  ON learning_materials (school_id, is_published, created_at DESC);
