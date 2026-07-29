CREATE TABLE IF NOT EXISTS locations (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  country TEXT NOT NULL DEFAULT 'Rwanda',
  province TEXT NOT NULL,
  district TEXT NOT NULL,
  sector TEXT NOT NULL,
  cell TEXT NOT NULL,
  village TEXT NOT NULL,
  province_order INT NOT NULL,
  district_order INT NOT NULL,
  sector_order INT NOT NULL,
  cell_order INT NOT NULL,
  village_order INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS locations_hierarchy_unique
  ON locations (
    lower(country),
    lower(province),
    lower(district),
    lower(sector),
    lower(cell),
    lower(village)
  );

CREATE INDEX IF NOT EXISTS locations_province_idx
  ON locations (country, province_order, province);

CREATE INDEX IF NOT EXISTS locations_district_idx
  ON locations (country, province, district_order, district);

CREATE INDEX IF NOT EXISTS locations_sector_idx
  ON locations (country, province, district, sector_order, sector);

CREATE INDEX IF NOT EXISTS locations_cell_idx
  ON locations (country, province, district, sector, cell_order, cell);
