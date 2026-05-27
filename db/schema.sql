CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

DO $$ BEGIN
  CREATE TYPE city_object_type AS ENUM (
    'school',
    'mall',
    'hospital',
    'stop',
    'station',
    'hub',
    'district'
  );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS city_objects (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  external_id TEXT,
  name TEXT NOT NULL,
  type city_object_type NOT NULL,
  lat DOUBLE PRECISION NOT NULL,
  lon DOUBLE PRECISION NOT NULL,
  address TEXT,
  source TEXT NOT NULL DEFAULT 'manual',
  geom GEOGRAPHY(POINT, 4326) GENERATED ALWAYS AS (
    ST_SetSRID(ST_MakePoint(lon, lat), 4326)::geography
  ) STORED,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS city_objects_geom_idx ON city_objects USING GIST (geom);
CREATE INDEX IF NOT EXISTS city_objects_type_idx ON city_objects (type);

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
  area TEXT NOT NULL,
  problem TEXT NOT NULL,
  recommendation TEXT NOT NULL,
  confidence NUMERIC(4,3) NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
  model_name TEXT NOT NULL DEFAULT 'rules-v1',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO accessibility_scores (district, avg_time_to_stop, avg_time_to_hub, score)
VALUES
  ('Алмалинский', 7, 14, 78),
  ('Ауэзовский', 13, 21, 54),
  ('Бостандыкский', 10, 18, 68),
  ('Наурызбайский', 19, 27, 31),
  ('Турксибский', 16, 24, 43),
  ('Медеуский', 12, 20, 62)
ON CONFLICT DO NOTHING;

INSERT INTO ai_recommendations (area, problem, recommendation, confidence)
VALUES
  (
    'Наурызбайский',
    'High activity density with weak access to trunk transport',
    'Evaluate an express route or dedicated feeder line for this area.',
    0.82
  ),
  (
    'Турксибский',
    'Long average time to transfer hub',
    'Consider a transfer hub near the strongest stop cluster.',
    0.74
  ),
  (
    'Ауэзовский',
    'Likely route duplication',
    'Audit overlapping routes and move capacity toward underserved corridors.',
    0.69
  )
ON CONFLICT DO NOTHING;

