ALTER TABLE schools
  ADD COLUMN IF NOT EXISTS accreditation_number TEXT,
  ADD COLUMN IF NOT EXISTS school_members TEXT,
  ADD COLUMN IF NOT EXISTS website TEXT,
  ADD COLUMN IF NOT EXISTS student_capacity INTEGER,
  ADD COLUMN IF NOT EXISTS uniform_required BOOLEAN,
  ADD COLUMN IF NOT EXISTS attendance_system TEXT,
  ADD COLUMN IF NOT EXISTS scholarship_available BOOLEAN,
  ADD COLUMN IF NOT EXISTS classrooms INTEGER,
  ADD COLUMN IF NOT EXISTS library BOOLEAN,
  ADD COLUMN IF NOT EXISTS online_classes BOOLEAN,
  ADD COLUMN IF NOT EXISTS contact_email TEXT,
  ADD COLUMN IF NOT EXISTS contact_phone TEXT,
  ADD COLUMN IF NOT EXISTS contact_alt_phone TEXT,
  ADD COLUMN IF NOT EXISTS contact_whatsapp TEXT,
  ADD COLUMN IF NOT EXISTS address_country TEXT,
  ADD COLUMN IF NOT EXISTS address_province TEXT,
  ADD COLUMN IF NOT EXISTS address_district TEXT,
  ADD COLUMN IF NOT EXISTS address_sector TEXT,
  ADD COLUMN IF NOT EXISTS address_cell TEXT,
  ADD COLUMN IF NOT EXISTS address_village TEXT,
  ADD COLUMN IF NOT EXISTS address_state TEXT,
  ADD COLUMN IF NOT EXISTS address_street TEXT,
  ADD COLUMN IF NOT EXISTS address_city TEXT,
  ADD COLUMN IF NOT EXISTS address_postal_code TEXT,
  ADD COLUMN IF NOT EXISTS address_google_map_url TEXT;

CREATE TABLE IF NOT EXISTS school_curricula (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  curriculum_id TEXT NOT NULL CHECK (char_length(curriculum_id) = 24),
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (school_id, curriculum_id)
);

CREATE INDEX IF NOT EXISTS school_curricula_school_idx ON school_curricula (school_id, position);

CREATE TABLE IF NOT EXISTS school_education_levels (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  education_level_id TEXT NOT NULL CHECK (char_length(education_level_id) = 24),
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (school_id, education_level_id)
);

CREATE INDEX IF NOT EXISTS school_education_levels_school_idx ON school_education_levels (school_id, position);

CREATE TABLE IF NOT EXISTS school_social_media (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  platform TEXT NOT NULL,
  url TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS school_social_media_school_idx ON school_social_media (school_id, position);

CREATE TABLE IF NOT EXISTS school_profile_values (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  value_type TEXT NOT NULL,
  value TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS school_profile_values_school_type_idx ON school_profile_values (school_id, value_type, position);
