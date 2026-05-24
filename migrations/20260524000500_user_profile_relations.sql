ALTER TABLE users
  ADD COLUMN IF NOT EXISTS bio TEXT,
  ADD COLUMN IF NOT EXISTS age_year INTEGER,
  ADD COLUMN IF NOT EXISTS age_month INTEGER,
  ADD COLUMN IF NOT EXISTS age_day INTEGER,
  ADD COLUMN IF NOT EXISTS dream_career TEXT,
  ADD COLUMN IF NOT EXISTS health_or_learning_notes TEXT,
  ADD COLUMN IF NOT EXISTS employment_type TEXT,
  ADD COLUMN IF NOT EXISTS teaching_start_date TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS years_of_experience TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS education_level TEXT,
  ADD COLUMN IF NOT EXISTS preferred_age_group TEXT,
  ADD COLUMN IF NOT EXISTS department TEXT,
  ADD COLUMN IF NOT EXISTS job_title TEXT,
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

CREATE TABLE IF NOT EXISTS user_background_images (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  image_id TEXT NOT NULL,
  url TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS user_background_images_user_idx ON user_background_images (user_id, position);

CREATE TABLE IF NOT EXISTS user_social_media (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  platform TEXT NOT NULL,
  url TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS user_social_media_user_idx ON user_social_media (user_id, position);

CREATE TABLE IF NOT EXISTS user_profile_values (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  value_type TEXT NOT NULL,
  value TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS user_profile_values_user_type_idx ON user_profile_values (user_id, value_type, position);
