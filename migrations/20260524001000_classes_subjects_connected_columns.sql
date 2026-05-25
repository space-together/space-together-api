ALTER TABLE classes
  ADD COLUMN IF NOT EXISTS class_teacher_id TEXT REFERENCES teachers(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS class_type TEXT NOT NULL DEFAULT 'Private',
  ADD COLUMN IF NOT EXISTS level_type TEXT,
  ADD COLUMN IF NOT EXISTS main_class_id TEXT CHECK (main_class_id IS NULL OR char_length(main_class_id) = 24),
  ADD COLUMN IF NOT EXISTS trade_id TEXT CHECK (trade_id IS NULL OR char_length(trade_id) = 24),
  ADD COLUMN IF NOT EXISTS image_id TEXT,
  ADD COLUMN IF NOT EXISTS image TEXT,
  ADD COLUMN IF NOT EXISTS description TEXT,
  ADD COLUMN IF NOT EXISTS capacity INTEGER,
  ADD COLUMN IF NOT EXISTS subject TEXT,
  ADD COLUMN IF NOT EXISTS grade_level TEXT;

CREATE INDEX IF NOT EXISTS classes_class_teacher_idx ON classes (class_teacher_id);
CREATE INDEX IF NOT EXISTS classes_main_class_idx ON classes (main_class_id);
CREATE INDEX IF NOT EXISTS classes_trade_idx ON classes (trade_id);
CREATE INDEX IF NOT EXISTS classes_school_type_idx ON classes (school_id, class_type);

CREATE TABLE IF NOT EXISTS class_tags (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  class_id TEXT NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
  tag TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (class_id, tag)
);

CREATE INDEX IF NOT EXISTS class_tags_class_idx ON class_tags (class_id, position);

CREATE TABLE IF NOT EXISTS class_background_images (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  class_id TEXT NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
  public_id TEXT NOT NULL,
  url TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS class_background_images_class_idx ON class_background_images (class_id, position);

CREATE TABLE IF NOT EXISTS class_settings (
  class_id TEXT PRIMARY KEY REFERENCES classes(id) ON DELETE CASCADE,
  auto_enroll_subclasses BOOLEAN,
  student_visibility TEXT,
  student_can_chat BOOLEAN,
  student_can_upload_homework BOOLEAN,
  student_can_comment BOOLEAN,
  student_can_view_all_students BOOLEAN,
  late_after_minutes INTEGER,
  required_attendance_percentage REAL,
  allow_resubmission BOOLEAN,
  max_late_days TEXT,
  teacher_can_edit_marks BOOLEAN,
  teacher_can_take_attendance BOOLEAN,
  teacher_can_remove_students BOOLEAN,
  teacher_visibility BOOLEAN,
  can_edit_class_info BOOLEAN,
  can_add_students BOOLEAN,
  can_remove_students BOOLEAN,
  can_manage_subjects BOOLEAN,
  can_manage_timetable BOOLEAN,
  can_approve_requests BOOLEAN,
  can_assign_roles BOOLEAN,
  can_send_parent_notifications BOOLEAN,
  can_add_teachers BOOLEAN,
  require_two_person_approval_for_results BOOLEAN,
  log_all_teacher_changes BOOLEAN,
  period_length_minutes INTEGER,
  periods_per_day INTEGER,
  prevent_double_teacher_booking BOOLEAN,
  prevent_duplicate_subject_same_day BOOLEAN,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER class_settings_set_updated_at BEFORE UPDATE ON class_settings
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS class_timetable_periods (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  class_id TEXT NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
  day_key TEXT NOT NULL,
  period INTEGER,
  subject TEXT,
  teacher_id TEXT CHECK (teacher_id IS NULL OR char_length(teacher_id) = 24),
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS class_timetable_periods_class_idx ON class_timetable_periods (class_id, day_key, position);

CREATE TABLE IF NOT EXISTS class_break_times (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  class_id TEXT NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
  start_time TEXT,
  end_time TEXT,
  label TEXT,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS class_break_times_class_idx ON class_break_times (class_id, position);

ALTER TABLE class_subjects
  ADD COLUMN IF NOT EXISTS teacher_id TEXT REFERENCES teachers(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS main_subject_id TEXT CHECK (main_subject_id IS NULL OR char_length(main_subject_id) = 24),
  ADD COLUMN IF NOT EXISTS description TEXT,
  ADD COLUMN IF NOT EXISTS estimated_hours INTEGER NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS credits INTEGER,
  ADD COLUMN IF NOT EXISTS created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS disable BOOLEAN;

CREATE UNIQUE INDEX IF NOT EXISTS class_subjects_school_code_unique ON class_subjects (school_id, lower(code)) WHERE code IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS class_subjects_teacher_idx ON class_subjects (teacher_id);
CREATE INDEX IF NOT EXISTS class_subjects_main_subject_idx ON class_subjects (main_subject_id);

CREATE TABLE IF NOT EXISTS class_subject_topics (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  class_subject_id TEXT NOT NULL REFERENCES class_subjects(id) ON DELETE CASCADE,
  parent_topic_id TEXT REFERENCES class_subject_topics(id) ON DELETE CASCADE,
  order_key TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT,
  estimated_hours INTEGER,
  credits INTEGER,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS class_subject_topics_subject_idx ON class_subject_topics (class_subject_id, parent_topic_id, position);
