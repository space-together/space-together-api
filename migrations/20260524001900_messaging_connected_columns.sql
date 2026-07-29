ALTER TABLE conversations
  ADD COLUMN IF NOT EXISTS is_group BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS name TEXT,
  ADD COLUMN IF NOT EXISTS encryption_key_version INTEGER NOT NULL DEFAULT 1;

UPDATE conversations
SET is_group = COALESCE(is_group, conversation_type = 'GROUP'),
    name = COALESCE(name, title)
WHERE name IS NULL OR conversation_type IS NOT NULL;

ALTER TABLE conversation_participants
  ADD COLUMN IF NOT EXISTS actor_id TEXT CHECK (actor_id IS NULL OR char_length(actor_id) = 24),
  ADD COLUMN IF NOT EXISTS actor_role TEXT;

ALTER TABLE conversation_participants
  DROP CONSTRAINT IF EXISTS conversation_participants_check;

UPDATE conversation_participants
SET actor_id = COALESCE(actor_id, user_id, student_id),
    actor_role = COALESCE(actor_role, role)
WHERE actor_id IS NULL OR actor_role IS NULL;

ALTER TABLE conversation_participants
  ADD CONSTRAINT conversation_participants_actor_check CHECK (
    actor_id IS NOT NULL OR user_id IS NOT NULL OR student_id IS NOT NULL
  );

ALTER TABLE conversation_keys
  ALTER COLUMN user_id DROP NOT NULL,
  ADD COLUMN IF NOT EXISTS actor_id TEXT CHECK (actor_id IS NULL OR char_length(actor_id) = 24),
  ADD COLUMN IF NOT EXISTS actor_role TEXT,
  ADD COLUMN IF NOT EXISTS encrypted_key_for_user TEXT;

UPDATE conversation_keys
SET actor_id = COALESCE(actor_id, user_id),
    actor_role = COALESCE(actor_role, user_role),
    encrypted_key_for_user = COALESCE(encrypted_key_for_user, encrypted_key)
WHERE actor_id IS NULL OR encrypted_key_for_user IS NULL;

ALTER TABLE messages
  ADD COLUMN IF NOT EXISTS sender_actor_id TEXT CHECK (sender_actor_id IS NULL OR char_length(sender_actor_id) = 24),
  ADD COLUMN IF NOT EXISTS sender_role TEXT,
  ADD COLUMN IF NOT EXISTS encrypted_payload TEXT,
  ADD COLUMN IF NOT EXISTS nonce TEXT,
  ADD COLUMN IF NOT EXISTS key_version INTEGER NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS file_url TEXT,
  ADD COLUMN IF NOT EXISTS file_public_id TEXT,
  ADD COLUMN IF NOT EXISTS client_message_id TEXT;

UPDATE messages
SET sender_actor_id = COALESCE(sender_actor_id, sender_user_id, sender_student_id),
    sender_role = COALESCE(sender_role, CASE WHEN sender_student_id IS NOT NULL THEN 'STUDENT' ELSE 'USER' END),
    encrypted_payload = COALESCE(encrypted_payload, body, ''),
    nonce = COALESCE(nonce, '')
WHERE sender_actor_id IS NULL OR encrypted_payload IS NULL OR nonce IS NULL;

CREATE TABLE IF NOT EXISTS message_read_receipts (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  actor_id TEXT NOT NULL CHECK (char_length(actor_id) = 24),
  actor_role TEXT NOT NULL,
  read_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (message_id, actor_id, actor_role)
);

CREATE UNIQUE INDEX IF NOT EXISTS messages_client_message_id_unique
  ON messages (client_message_id)
  WHERE client_message_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS conversation_participants_actor_idx
  ON conversation_participants (actor_id, actor_role, left_at);
CREATE INDEX IF NOT EXISTS conversation_participants_conversation_idx
  ON conversation_participants (conversation_id, left_at);
CREATE UNIQUE INDEX IF NOT EXISTS conversation_participants_conversation_actor_unique
  ON conversation_participants (conversation_id, actor_id, actor_role)
  WHERE actor_id IS NOT NULL AND left_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS conversation_keys_conversation_actor_unique
  ON conversation_keys (conversation_id, actor_id, actor_role)
  WHERE actor_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS messages_sender_actor_idx
  ON messages (sender_actor_id, sender_role, created_at DESC);
CREATE INDEX IF NOT EXISTS message_read_receipts_message_idx
  ON message_read_receipts (message_id);
