ALTER TABLE teachers
  ADD COLUMN IF NOT EXISTS creator_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS name TEXT,
  ADD COLUMN IF NOT EXISTS email TEXT,
  ADD COLUMN IF NOT EXISTS phone TEXT,
  ADD COLUMN IF NOT EXISTS gender TEXT,
  ADD COLUMN IF NOT EXISTS image TEXT,
  ADD COLUMN IF NOT EXISTS image_id TEXT,
  ADD COLUMN IF NOT EXISTS teacher_type TEXT NOT NULL DEFAULT 'Regular',
  ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

CREATE INDEX IF NOT EXISTS teachers_school_email_idx ON teachers (school_id, lower(email)) WHERE email IS NOT NULL;
CREATE INDEX IF NOT EXISTS teachers_creator_id_idx ON teachers (creator_id);
CREATE INDEX IF NOT EXISTS teachers_type_idx ON teachers (teacher_type);

CREATE TABLE IF NOT EXISTS teacher_classes (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  teacher_id TEXT NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
  class_id TEXT NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (teacher_id, class_id)
);

CREATE TABLE IF NOT EXISTS teacher_subjects (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  teacher_id TEXT NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
  class_subject_id TEXT NOT NULL REFERENCES class_subjects(id) ON DELETE CASCADE,
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (teacher_id, class_subject_id)
);

CREATE TABLE IF NOT EXISTS teacher_tags (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  teacher_id TEXT NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
  tag TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (teacher_id, tag)
);

CREATE INDEX IF NOT EXISTS teacher_classes_teacher_idx ON teacher_classes (teacher_id, position);
CREATE INDEX IF NOT EXISTS teacher_subjects_teacher_idx ON teacher_subjects (teacher_id, position);
CREATE INDEX IF NOT EXISTS teacher_tags_teacher_idx ON teacher_tags (teacher_id, position);

ALTER TABLE school_staff
  ADD COLUMN IF NOT EXISTS creator_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS name TEXT,
  ADD COLUMN IF NOT EXISTS email TEXT,
  ADD COLUMN IF NOT EXISTS staff_type TEXT NOT NULL DEFAULT 'HeadOfStudies',
  ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true,
  ADD COLUMN IF NOT EXISTS image TEXT,
  ADD COLUMN IF NOT EXISTS image_id TEXT;

CREATE INDEX IF NOT EXISTS school_staff_school_email_idx ON school_staff (school_id, lower(email)) WHERE email IS NOT NULL;
CREATE INDEX IF NOT EXISTS school_staff_creator_id_idx ON school_staff (creator_id);
CREATE INDEX IF NOT EXISTS school_staff_type_idx ON school_staff (staff_type);

CREATE TABLE IF NOT EXISTS school_staff_tags (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  staff_id TEXT NOT NULL REFERENCES school_staff(id) ON DELETE CASCADE,
  tag TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (staff_id, tag)
);

CREATE INDEX IF NOT EXISTS school_staff_tags_staff_idx ON school_staff_tags (staff_id, position);

ALTER TABLE roles
  ADD COLUMN IF NOT EXISTS role_type TEXT NOT NULL DEFAULT 'Custom',
  ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

ALTER TABLE user_role_assignments
  ADD COLUMN IF NOT EXISTS role_id TEXT REFERENCES roles(id) ON DELETE CASCADE,
  ADD COLUMN IF NOT EXISTS assigned_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS user_role_assignments_role_id_idx ON user_role_assignments (role_id);
CREATE UNIQUE INDEX IF NOT EXISTS user_role_assignments_user_role_school_unique
  ON user_role_assignments (user_id, role_id, school_id)
  WHERE role_id IS NOT NULL AND school_id IS NOT NULL;
