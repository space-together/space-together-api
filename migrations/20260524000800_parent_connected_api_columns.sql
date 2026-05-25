ALTER TABLE parents
  ADD COLUMN IF NOT EXISTS name TEXT,
  ADD COLUMN IF NOT EXISTS email TEXT,
  ADD COLUMN IF NOT EXISTS phone TEXT,
  ADD COLUMN IF NOT EXISTS gender TEXT,
  ADD COLUMN IF NOT EXISTS image TEXT,
  ADD COLUMN IF NOT EXISTS image_id TEXT,
  ADD COLUMN IF NOT EXISTS occupation TEXT,
  ADD COLUMN IF NOT EXISTS national_id TEXT,
  ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

CREATE INDEX IF NOT EXISTS parents_school_email_idx ON parents (school_id, lower(email)) WHERE email IS NOT NULL;
CREATE INDEX IF NOT EXISTS parents_status_idx ON parents (status);
CREATE INDEX IF NOT EXISTS parent_student_links_school_idx ON parent_student_links (school_id, status);
CREATE INDEX IF NOT EXISTS parent_student_links_parent_student_idx ON parent_student_links (parent_id, student_id);
