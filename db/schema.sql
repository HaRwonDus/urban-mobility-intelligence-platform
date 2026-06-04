CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

DO $$ BEGIN
  CREATE TYPE city_object_type AS ENUM (
    'school',
    'mall',
    'hospital',
    'stop',
    'metro',
    'station',
    'hub',
    'bus_station',
    'airport',
    'university',
    'district'
  );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END $$;

ALTER TYPE city_object_type ADD VALUE IF NOT EXISTS 'metro';
ALTER TYPE city_object_type ADD VALUE IF NOT EXISTS 'bus_station';
ALTER TYPE city_object_type ADD VALUE IF NOT EXISTS 'airport';
ALTER TYPE city_object_type ADD VALUE IF NOT EXISTS 'university';

CREATE TABLE IF NOT EXISTS districts (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL UNIQUE,
  slug TEXT NOT NULL UNIQUE,
  lat DOUBLE PRECISION NOT NULL,
  lon DOUBLE PRECISION NOT NULL,
  population INTEGER,
  boundary GEOMETRY(POLYGON, 4326),
  geom GEOGRAPHY(POINT, 4326) GENERATED ALWAYS AS (
    ST_SetSRID(ST_MakePoint(lon, lat), 4326)::geography
  ) STORED,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE districts
  ADD COLUMN IF NOT EXISTS boundary GEOMETRY(POLYGON, 4326);

CREATE INDEX IF NOT EXISTS districts_geom_idx ON districts USING GIST (geom);
CREATE INDEX IF NOT EXISTS districts_boundary_idx ON districts USING GIST (boundary);

CREATE TABLE IF NOT EXISTS city_objects (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  external_id TEXT,
  name TEXT NOT NULL,
  type city_object_type NOT NULL,
  lat DOUBLE PRECISION NOT NULL,
  lon DOUBLE PRECISION NOT NULL,
  address TEXT,
  district_id UUID REFERENCES districts(id) ON DELETE SET NULL,
  source TEXT NOT NULL DEFAULT 'manual',
  raw JSONB NOT NULL DEFAULT '{}'::jsonb,
  geom GEOGRAPHY(POINT, 4326) GENERATED ALWAYS AS (
    ST_SetSRID(ST_MakePoint(lon, lat), 4326)::geography
  ) STORED,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (source, external_id)
);

ALTER TABLE city_objects
  ADD COLUMN IF NOT EXISTS district_id UUID REFERENCES districts(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS raw JSONB NOT NULL DEFAULT '{}'::jsonb,
  ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT now();

DO $$ BEGIN
  ALTER TABLE city_objects ADD CONSTRAINT city_objects_source_external_id_key UNIQUE (source, external_id);
EXCEPTION
  WHEN duplicate_object THEN NULL;
END $$;

CREATE INDEX IF NOT EXISTS city_objects_geom_idx ON city_objects USING GIST (geom);
CREATE INDEX IF NOT EXISTS city_objects_type_idx ON city_objects (type);
CREATE INDEX IF NOT EXISTS city_objects_district_idx ON city_objects (district_id);

CREATE TABLE IF NOT EXISTS transport_stops (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  city_object_id UUID NOT NULL UNIQUE REFERENCES city_objects(id) ON DELETE CASCADE,
  stop_kind TEXT NOT NULL,
  route_count INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS routes (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  origin_id UUID REFERENCES city_objects(id) ON DELETE SET NULL,
  destination_id UUID REFERENCES city_objects(id) ON DELETE SET NULL,
  distance_m INTEGER NOT NULL,
  duration_sec INTEGER NOT NULL,
  transport_type TEXT NOT NULL,
  source TEXT NOT NULL DEFAULT '2gis',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS mobility_scores (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  district_id UUID NOT NULL REFERENCES districts(id) ON DELETE CASCADE,
  avg_time_to_stop_min NUMERIC(7,2) NOT NULL,
  avg_time_to_hub_min NUMERIC(7,2) NOT NULL,
  poi_density NUMERIC(8,3) NOT NULL,
  connectivity_score NUMERIC(5,2) NOT NULL,
  score NUMERIC(5,2) NOT NULL CHECK (score >= 0 AND score <= 100),
  calculated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS mobility_scores_district_idx
  ON mobility_scores (district_id, calculated_at DESC);

CREATE TABLE IF NOT EXISTS accessibility_scores (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  district TEXT NOT NULL,
  avg_time_to_stop INTEGER NOT NULL,
  avg_time_to_hub INTEGER NOT NULL,
  score NUMERIC(5,2) NOT NULL CHECK (score >= 0 AND score <= 100),
  calculated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS accessibility_scores_district_idx
  ON accessibility_scores (district, calculated_at DESC);

CREATE TABLE IF NOT EXISTS ai_recommendations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  district_id UUID REFERENCES districts(id) ON DELETE CASCADE,
  area TEXT NOT NULL,
  problem TEXT NOT NULL,
  recommendation TEXT NOT NULL,
  confidence NUMERIC(4,3) NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
  model_name TEXT NOT NULL DEFAULT 'rules-v1',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE ai_recommendations
  ADD COLUMN IF NOT EXISTS district_id UUID REFERENCES districts(id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS ai_recommendations_district_idx
  ON ai_recommendations (district_id, created_at DESC);

CREATE TABLE IF NOT EXISTS sync_logs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  provider TEXT NOT NULL DEFAULT '2gis',
  status TEXT NOT NULL,
  objects_loaded INTEGER NOT NULL DEFAULT 0,
  districts_updated INTEGER NOT NULL DEFAULT 0,
  message TEXT,
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  finished_at TIMESTAMPTZ
);

INSERT INTO districts (name, slug, lat, lon, population)
VALUES
  ('Almalinsky', 'almalinsky', 43.2489, 76.9286, 220000),
  ('Auezovsky', 'auezovsky', 43.2327, 76.8477, 310000),
  ('Bostandyk', 'bostandyk', 43.2034, 76.9067, 360000),
  ('Nauryzbay', 'nauryzbay', 43.1972, 76.7825, 160000),
  ('Turksib', 'turksib', 43.3335, 76.9870, 235000),
  ('Medeu', 'medeu', 43.2244, 76.9958, 230000),
  ('Zhetysu', 'zhetysu', 43.2901, 76.9350, 170000),
  ('Alatau', 'alatau', 43.3006, 76.8287, 290000)
ON CONFLICT (slug) DO UPDATE SET
  name = EXCLUDED.name,
  lat = EXCLUDED.lat,
  lon = EXCLUDED.lon,
  population = EXCLUDED.population;

UPDATE districts
SET boundary = ST_Buffer(geom, 5500)::geometry
WHERE boundary IS NULL;

INSERT INTO city_objects (external_id, name, type, lat, lon, address, source, raw)
VALUES
  (
    'seed:airport:almaty-international',
    'Almaty International Airport',
    'airport',
    43.3521,
    77.0405,
    'Mailin St, Almaty',
    'seed',
    '{"avg_daily_arrivals": 11500, "avg_daily_departures": 11600, "hub_role": "air gateway"}'::jsonb
  ),
  (
    'seed:rail:almaty-1',
    'Almaty-1 Railway Station',
    'station',
    43.3417,
    76.9398,
    'Almaty-1',
    'seed',
    '{"avg_daily_arrivals": 8200, "avg_daily_departures": 7900, "hub_role": "rail gateway"}'::jsonb
  ),
  (
    'seed:rail:almaty-2',
    'Almaty-2 Railway Station',
    'station',
    43.2638,
    76.9455,
    'Abylai Khan Ave',
    'seed',
    '{"avg_daily_arrivals": 9800, "avg_daily_departures": 10100, "hub_role": "rail gateway"}'::jsonb
  ),
  (
    'seed:bus:sairan',
    'Sairan Bus Station',
    'bus_station',
    43.2398,
    76.8506,
    'Tole Bi St',
    'seed',
    '{"avg_daily_arrivals": 4300, "avg_daily_departures": 4500, "hub_role": "regional bus gateway"}'::jsonb
  )
ON CONFLICT (source, external_id) DO UPDATE SET
  name = EXCLUDED.name,
  type = EXCLUDED.type,
  lat = EXCLUDED.lat,
  lon = EXCLUDED.lon,
  address = EXCLUDED.address,
  raw = EXCLUDED.raw,
  updated_at = now();

INSERT INTO transport_stops (city_object_id, stop_kind, route_count)
SELECT id, type::text, 0
FROM city_objects
WHERE type::text IN ('airport', 'station', 'bus_station')
ON CONFLICT (city_object_id) DO UPDATE SET stop_kind = EXCLUDED.stop_kind;
