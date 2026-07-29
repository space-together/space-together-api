-- User public keys for end-to-end encryption
CREATE TABLE IF NOT EXISTS user_public_keys (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  user_id TEXT NOT NULL,
  public_key TEXT NOT NULL,
  key_algorithm TEXT NOT NULL DEFAULT 'RSA-2048',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS user_public_keys_user_id_unique ON user_public_keys (user_id);
CREATE TRIGGER user_public_keys_set_updated_at BEFORE UPDATE ON user_public_keys
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Template subjects: global subject catalog
CREATE TABLE IF NOT EXISTS template_subjects (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  name TEXT NOT NULL,
  code TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  category TEXT NOT NULL DEFAULT 'Other',
  estimated_hours INT NOT NULL DEFAULT 0,
  credits INT,
  prerequisites JSONB NOT NULL DEFAULT '[]',
  topics JSONB NOT NULL DEFAULT '[]',
  created_by TEXT,
  disable BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS template_subjects_code_unique ON template_subjects (lower(code));
CREATE INDEX IF NOT EXISTS template_subjects_category_idx ON template_subjects (category);
CREATE INDEX IF NOT EXISTS template_subjects_disable_idx ON template_subjects (disable);
CREATE TRIGGER template_subjects_set_updated_at BEFORE UPDATE ON template_subjects
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- School timetables: JSONB for complex nested schedule data
CREATE TABLE IF NOT EXISTS school_timetables (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL CHECK (char_length(school_id) = 24),
  academic_year_id TEXT,
  default_weekly_schedule JSONB NOT NULL DEFAULT '[]',
  overrides JSONB NOT NULL DEFAULT '[]',
  events JSONB NOT NULL DEFAULT '[]',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS school_timetables_school_year_unique
  ON school_timetables (school_id, academic_year_id);
CREATE INDEX IF NOT EXISTS school_timetables_school_id_idx ON school_timetables (school_id);
CREATE TRIGGER school_timetables_set_updated_at BEFORE UPDATE ON school_timetables
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Class timetables: JSONB for complex nested weekly schedule
CREATE TABLE IF NOT EXISTS class_timetables (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  class_id TEXT NOT NULL CHECK (char_length(class_id) = 24),
  education_year_id TEXT NOT NULL CHECK (char_length(education_year_id) = 24),
  term_order INT NOT NULL DEFAULT 1,
  weekly_schedule JSONB NOT NULL DEFAULT '[]',
  disabled BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS class_timetables_class_year_unique
  ON class_timetables (class_id, education_year_id);
CREATE INDEX IF NOT EXISTS class_timetables_class_id_idx ON class_timetables (class_id);
CREATE INDEX IF NOT EXISTS class_timetables_education_year_id_idx ON class_timetables (education_year_id);
CREATE TRIGGER class_timetables_set_updated_at BEFORE UPDATE ON class_timetables
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Join school requests
CREATE TABLE IF NOT EXISTS join_school_requests (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL CHECK (char_length(school_id) = 24),
  invited_user_id TEXT,
  class_id TEXT,
  role TEXT NOT NULL,
  email TEXT NOT NULL,
  type TEXT NOT NULL DEFAULT '',
  message TEXT,
  status TEXT NOT NULL DEFAULT 'Pending',
  sent_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  responded_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  sent_by TEXT NOT NULL CHECK (char_length(sent_by) = 24),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS join_school_requests_email_school_status_idx
  ON join_school_requests (email, school_id, status);
CREATE INDEX IF NOT EXISTS join_school_requests_school_id_idx ON join_school_requests (school_id);
CREATE INDEX IF NOT EXISTS join_school_requests_status_idx ON join_school_requests (status);
CREATE INDEX IF NOT EXISTS join_school_requests_expires_at_idx ON join_school_requests (expires_at);
CREATE TRIGGER join_school_requests_set_updated_at BEFORE UPDATE ON join_school_requests
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- School backups: metadata only; actual files managed on disk
CREATE TABLE IF NOT EXISTS school_backups (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT,
  backup_name TEXT NOT NULL,
  backup_type TEXT NOT NULL DEFAULT 'Manual',
  file_path TEXT NOT NULL DEFAULT '',
  size_bytes BIGINT NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'InProgress',
  created_by TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  completed_at TIMESTAMPTZ,
  error_message TEXT
);
CREATE INDEX IF NOT EXISTS school_backups_school_id_idx ON school_backups (school_id);
CREATE INDEX IF NOT EXISTS school_backups_status_idx ON school_backups (status);
CREATE INDEX IF NOT EXISTS school_backups_created_at_idx ON school_backups (school_id, created_at DESC);
