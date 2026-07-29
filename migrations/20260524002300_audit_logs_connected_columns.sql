ALTER TABLE audit_logs
  ADD COLUMN IF NOT EXISTS actor_role TEXT,
  ADD COLUMN IF NOT EXISTS severity TEXT NOT NULL DEFAULT 'INFO',
  ADD COLUMN IF NOT EXISTS ip_address TEXT,
  ADD COLUMN IF NOT EXISTS user_agent TEXT;

UPDATE audit_logs
SET actor_role = COALESCE(actor_role, 'STUDENT'),
    severity = COALESCE(severity, 'INFO')
WHERE actor_role IS NULL OR severity IS NULL;

CREATE INDEX IF NOT EXISTS audit_logs_actor_role_created_idx
  ON audit_logs (actor_user_id, actor_role, created_at DESC);
CREATE INDEX IF NOT EXISTS audit_logs_action_created_idx
  ON audit_logs (action, created_at DESC);
CREATE INDEX IF NOT EXISTS audit_logs_severity_created_idx
  ON audit_logs (severity, created_at DESC);
