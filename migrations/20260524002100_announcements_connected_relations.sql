ALTER TABLE announcements
  ADD COLUMN IF NOT EXISTS published_actor_id TEXT CHECK (published_actor_id IS NULL OR char_length(published_actor_id) = 24),
  ADD COLUMN IF NOT EXISTS published_role TEXT;

UPDATE announcements
SET published_actor_id = COALESCE(published_actor_id, author_user_id),
    published_role = COALESCE(published_role, 'USER')
WHERE published_actor_id IS NULL OR published_role IS NULL;

CREATE TABLE IF NOT EXISTS announcement_classes (
  announcement_id TEXT NOT NULL REFERENCES announcements(id) ON DELETE CASCADE,
  class_id TEXT NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (announcement_id, class_id)
);

CREATE TABLE IF NOT EXISTS announcement_mentions (
  announcement_id TEXT NOT NULL REFERENCES announcements(id) ON DELETE CASCADE,
  actor_id TEXT NOT NULL CHECK (char_length(actor_id) = 24),
  actor_role TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (announcement_id, actor_id, actor_role)
);

INSERT INTO announcement_classes (announcement_id, class_id)
SELECT id, class_id
FROM announcements
WHERE class_id IS NOT NULL
ON CONFLICT DO NOTHING;

CREATE INDEX IF NOT EXISTS announcements_published_actor_idx
  ON announcements (published_actor_id, published_role, created_at DESC);
CREATE INDEX IF NOT EXISTS announcement_classes_class_idx
  ON announcement_classes (class_id, announcement_id);
CREATE INDEX IF NOT EXISTS announcement_mentions_actor_idx
  ON announcement_mentions (actor_id, actor_role, announcement_id);
