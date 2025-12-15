-- Database initialization script for Cassiopeia project

-- ISS tracking data
CREATE TABLE IF NOT EXISTS iss_fetch_log (
    id BIGSERIAL PRIMARY KEY,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source_url TEXT NOT NULL,
    payload JSONB NOT NULL
);

-- Indexes for ISS data
CREATE INDEX IF NOT EXISTS idx_iss_fetched_at ON iss_fetch_log(fetched_at DESC);
CREATE INDEX IF NOT EXISTS idx_iss_payload_gin ON iss_fetch_log USING GIN(payload);

-- OSDR (NASA Open Science Data Repository) items
CREATE TABLE IF NOT EXISTS osdr_items (
    id BIGSERIAL PRIMARY KEY,
    dataset_id TEXT,
    title TEXT,
    status TEXT,
    updated_at TIMESTAMPTZ,
    inserted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    raw JSONB NOT NULL
);

-- Unique constraint and indexes for OSDR
CREATE UNIQUE INDEX IF NOT EXISTS ux_osdr_dataset_id
    ON osdr_items(dataset_id) WHERE dataset_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_osdr_inserted_at ON osdr_items(inserted_at DESC);
CREATE INDEX IF NOT EXISTS idx_osdr_status ON osdr_items(status) WHERE status IS NOT NULL;

-- Space cache (unified cache for APOD, NEO, DONKI, SpaceX data)
CREATE TABLE IF NOT EXISTS space_cache (
    id BIGSERIAL PRIMARY KEY,
    source TEXT NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    payload JSONB NOT NULL
);

-- Indexes for space cache
CREATE INDEX IF NOT EXISTS idx_space_cache_source_time ON space_cache(source, fetched_at DESC);
CREATE INDEX IF NOT EXISTS idx_space_cache_payload_gin ON space_cache USING GIN(payload);

-- Telemetry legacy data (from Pascal/Rust generator)
CREATE TABLE IF NOT EXISTS telemetry_legacy (
    id BIGSERIAL PRIMARY KEY,
    recorded_at TIMESTAMPTZ NOT NULL,
    voltage NUMERIC(6,2) NOT NULL,
    temp NUMERIC(6,2) NOT NULL,
    source_file TEXT NOT NULL
);

-- Index for telemetry
CREATE INDEX IF NOT EXISTS idx_telemetry_recorded_at ON telemetry_legacy(recorded_at DESC);

-- CMS pages for frontend
CREATE TABLE IF NOT EXISTS cms_pages (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL
);

-- Seed CMS data
INSERT INTO cms_pages(slug, title, body)
VALUES
('welcome', 'Добро пожаловать', '<h3>Демо контент</h3><p>Этот текст хранится в БД</p>'),
('unsafe', 'Небезопасный пример', '<script>console.log("XSS training")</script><p>Если вы видите всплывашку значит защита не работает</p>')
ON CONFLICT (slug) DO NOTHING;

-- Performance optimization settings
-- Note: These are recommendations, actual tuning depends on hardware
COMMENT ON TABLE iss_fetch_log IS 'ISS position tracking data with JSON payloads';
COMMENT ON TABLE osdr_items IS 'NASA OSDR dataset items with upsert capability';
COMMENT ON TABLE space_cache IS 'Unified cache for multiple space APIs (APOD, NEO, DONKI, SpaceX)';
COMMENT ON TABLE telemetry_legacy IS 'Telemetry data from legacy generator service';
