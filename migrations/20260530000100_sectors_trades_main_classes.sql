-- Sectors: global education catalog (no school scope)
CREATE TABLE IF NOT EXISTS sectors (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  name TEXT NOT NULL,
  logo TEXT,
  logo_id TEXT,
  username TEXT NOT NULL,
  description TEXT,
  curriculum_start INT,
  curriculum_end INT,
  country TEXT NOT NULL,
  type TEXT NOT NULL,
  disable BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS sectors_name_lower_unique ON sectors (lower(name));
CREATE UNIQUE INDEX IF NOT EXISTS sectors_username_lower_unique ON sectors (lower(username));
CREATE INDEX IF NOT EXISTS sectors_country_type_idx ON sectors (country, type);
CREATE TRIGGER sectors_set_updated_at BEFORE UPDATE ON sectors
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Trades: linked to sectors, self-referential parent_trade
CREATE TABLE IF NOT EXISTS trades (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  sector_id TEXT REFERENCES sectors(id) ON DELETE SET NULL,
  trade_id TEXT REFERENCES trades(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  username TEXT NOT NULL,
  description TEXT,
  class_min INT NOT NULL DEFAULT 0,
  class_max INT NOT NULL DEFAULT 0,
  type TEXT NOT NULL DEFAULT 'SENIOR',
  disable BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS trades_name_lower_unique ON trades (lower(name));
CREATE UNIQUE INDEX IF NOT EXISTS trades_username_lower_unique ON trades (lower(username));
CREATE INDEX IF NOT EXISTS trades_sector_id_idx ON trades (sector_id);
CREATE INDEX IF NOT EXISTS trades_trade_id_idx ON trades (trade_id);
CREATE TRIGGER trades_set_updated_at BEFORE UPDATE ON trades
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Main classes: linked to trades, represent grade levels
CREATE TABLE IF NOT EXISTS main_classes (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  name TEXT NOT NULL,
  username TEXT NOT NULL,
  trade_id TEXT REFERENCES trades(id) ON DELETE SET NULL,
  level INT,
  description TEXT,
  disable BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS main_classes_name_lower_unique ON main_classes (lower(name));
CREATE UNIQUE INDEX IF NOT EXISTS main_classes_username_lower_unique ON main_classes (lower(username));
CREATE INDEX IF NOT EXISTS main_classes_trade_id_level_idx ON main_classes (trade_id, level);
CREATE TRIGGER main_classes_set_updated_at BEFORE UPDATE ON main_classes
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- School features: per-school feature flag registry
CREATE TABLE IF NOT EXISTS school_features (
  school_id TEXT NOT NULL CHECK (char_length(school_id) = 24),
  feature_name TEXT NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT true,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (school_id, feature_name)
);
CREATE INDEX IF NOT EXISTS school_features_school_id_idx ON school_features (school_id);
