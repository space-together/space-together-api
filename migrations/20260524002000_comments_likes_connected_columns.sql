ALTER TABLE comments
  ADD COLUMN IF NOT EXISTS author_actor_id TEXT CHECK (author_actor_id IS NULL OR char_length(author_actor_id) = 24),
  ADD COLUMN IF NOT EXISTS author_role TEXT,
  ADD COLUMN IF NOT EXISTS content TEXT,
  ADD COLUMN IF NOT EXISTS target_post_id TEXT CHECK (target_post_id IS NULL OR char_length(target_post_id) = 24);

UPDATE comments
SET author_actor_id = COALESCE(author_actor_id, author_user_id),
    author_role = COALESCE(author_role, 'USER'),
    content = COALESCE(content, body),
    target_post_id = COALESCE(target_post_id, target_id)
WHERE author_actor_id IS NULL
   OR content IS NULL
   OR target_post_id IS NULL;

CREATE INDEX IF NOT EXISTS comments_author_actor_idx ON comments (author_actor_id, author_role, created_at DESC);
CREATE INDEX IF NOT EXISTS comments_target_post_idx ON comments (target_post_id, created_at DESC);

ALTER TABLE likes
  ALTER COLUMN user_id DROP NOT NULL,
  ADD COLUMN IF NOT EXISTS actor_id TEXT CHECK (actor_id IS NULL OR char_length(actor_id) = 24),
  ADD COLUMN IF NOT EXISTS actor_role TEXT,
  ADD COLUMN IF NOT EXISTS like_value TEXT,
  ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

UPDATE likes
SET actor_id = COALESCE(actor_id, user_id),
    actor_role = COALESCE(actor_role, 'USER'),
    like_value = COALESCE(like_value, 'like')
WHERE actor_id IS NULL OR actor_role IS NULL OR like_value IS NULL;

DROP INDEX IF EXISTS likes_target_actor_unique;
CREATE UNIQUE INDEX IF NOT EXISTS likes_target_actor_unique
  ON likes (target_type, target_id, actor_id, actor_role)
  WHERE actor_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS likes_actor_idx ON likes (actor_id, actor_role);
