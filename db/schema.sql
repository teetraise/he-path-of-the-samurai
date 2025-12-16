-- PostgreSQL Schema для проекта Кассиопея
-- База данных: cassiopeia
-- Пользователь: cassiopeia_user

-- ============================================
-- 1. ISS FETCH LOG - Журнал запросов к API МКС
-- ============================================

CREATE TABLE IF NOT EXISTS iss_fetch_log (
    id BIGSERIAL PRIMARY KEY,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source_url TEXT NOT NULL,
    payload JSONB NOT NULL,

    -- Метаданные для быстрого поиска
    CONSTRAINT iss_fetch_log_payload_check CHECK (payload IS NOT NULL)
);

-- Индекс для сортировки по времени (последние запросы)
CREATE INDEX IF NOT EXISTS idx_iss_fetch_log_fetched_at
    ON iss_fetch_log (fetched_at DESC);

-- GIN индекс для поиска по JSONB полям
CREATE INDEX IF NOT EXISTS idx_iss_fetch_log_payload
    ON iss_fetch_log USING GIN (payload);

COMMENT ON TABLE iss_fetch_log IS 'История всех запросов к API МКС';
COMMENT ON COLUMN iss_fetch_log.payload IS 'Полный JSON ответ от API';
COMMENT ON COLUMN iss_fetch_log.source_url IS 'URL источника данных';

-- ============================================
-- 2. TELEMETRY LEGACY - Синтетическая телеметрия
-- ============================================

CREATE TABLE IF NOT EXISTS telemetry_legacy (
    id BIGSERIAL PRIMARY KEY,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    voltage DOUBLE PRECISION,
    temp DOUBLE PRECISION,
    source_file TEXT,

    -- Валидация данных
    CONSTRAINT telemetry_voltage_check CHECK (voltage > 0 AND voltage < 50),
    CONSTRAINT telemetry_temp_check CHECK (temp > -100 AND temp < 100)
);

-- Индекс для временных рядов
CREATE INDEX IF NOT EXISTS idx_telemetry_recorded_at
    ON telemetry_legacy (recorded_at DESC);

-- Индекс для поиска по source_file
CREATE INDEX IF NOT EXISTS idx_telemetry_source_file
    ON telemetry_legacy (source_file);

COMMENT ON TABLE telemetry_legacy IS 'Синтетическая телеметрия космических систем';
COMMENT ON COLUMN telemetry_legacy.voltage IS 'Напряжение в вольтах (V)';
COMMENT ON COLUMN telemetry_legacy.temp IS 'Температура в градусах Цельсия (°C)';

-- ============================================
-- 3. OSDR STUDIES - NASA Open Science Data Repository
-- ============================================

CREATE TABLE IF NOT EXISTS osdr_studies (
    id BIGSERIAL PRIMARY KEY,
    study_id TEXT UNIQUE NOT NULL,
    title TEXT,
    description TEXT,
    payload JSONB,
    synced_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT osdr_studies_study_id_check CHECK (study_id <> '')
);

-- Уникальный индекс по study_id
CREATE UNIQUE INDEX IF NOT EXISTS idx_osdr_studies_study_id
    ON osdr_studies (study_id);

-- Индекс для сортировки по времени синхронизации
CREATE INDEX IF NOT EXISTS idx_osdr_studies_synced_at
    ON osdr_studies (synced_at DESC);

-- GIN индекс для JSONB
CREATE INDEX IF NOT EXISTS idx_osdr_studies_payload
    ON osdr_studies USING GIN (payload);

COMMENT ON TABLE osdr_studies IS 'Исследования из NASA Open Science Data Repository';
COMMENT ON COLUMN osdr_studies.study_id IS 'Уникальный ID исследования в OSDR';
COMMENT ON COLUMN osdr_studies.payload IS 'Полные метаданные исследования';

-- ============================================
-- 4. SPACE SOURCES - Агрегированные космические данные
-- ============================================

CREATE TABLE IF NOT EXISTS space_sources (
    id BIGSERIAL PRIMARY KEY,
    source TEXT NOT NULL CHECK (source IN ('nasa_neo', 'apod', 'spacex', 'jwst')),
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    payload JSONB NOT NULL
);

-- Композитный индекс для быстрого поиска последних данных по источнику
CREATE INDEX IF NOT EXISTS idx_space_sources_source_fetched
    ON space_sources (source, fetched_at DESC);

-- GIN индекс для JSONB
CREATE INDEX IF NOT EXISTS idx_space_sources_payload
    ON space_sources USING GIN (payload);

COMMENT ON TABLE space_sources IS 'Агрегированные данные из внешних космических API';
COMMENT ON COLUMN space_sources.source IS 'Имя источника: nasa_neo, apod, spacex, jwst';
COMMENT ON COLUMN space_sources.payload IS 'JSON данные от API';

-- ============================================
-- 5. CMS BLOCKS - Управление контентом (Laravel)
-- ============================================

CREATE TABLE IF NOT EXISTS cms_blocks (
    id BIGSERIAL PRIMARY KEY,
    slug VARCHAR(255) UNIQUE NOT NULL,
    content TEXT NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,

    CONSTRAINT cms_blocks_slug_check CHECK (slug <> '')
);

-- Индекс для быстрого поиска по slug
CREATE UNIQUE INDEX IF NOT EXISTS idx_cms_blocks_slug
    ON cms_blocks (slug);

-- Индекс для фильтрации активных блоков
CREATE INDEX IF NOT EXISTS idx_cms_blocks_is_active
    ON cms_blocks (is_active) WHERE is_active = TRUE;

COMMENT ON TABLE cms_blocks IS 'CMS блоки для текстового контента';
COMMENT ON COLUMN cms_blocks.slug IS 'Уникальный идентификатор блока';

-- Вставка демо-данных
INSERT INTO cms_blocks (slug, content, is_active, created_at, updated_at)
VALUES
    ('welcome', 'Добро пожаловать в проект Кассиопея — систему отслеживания космических данных в реальном времени.', TRUE, NOW(), NOW()),
    ('iss_info', 'Международная космическая станция движется со скоростью ~27,600 км/ч на высоте ~400 км.', TRUE, NOW(), NOW()),
    ('about', 'Проект собирает данные из NASA, SpaceX, AstronomyAPI и других источников.', TRUE, NOW(), NOW())
ON CONFLICT (slug) DO NOTHING;

-- ============================================
-- 6. LARAVEL STANDARD TABLES
-- ============================================

-- Users table (Laravel authentication)
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    email_verified_at TIMESTAMP,
    password VARCHAR(255) NOT NULL,
    remember_token VARCHAR(100),
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users (email);

-- Cache table (Laravel cache)
CREATE TABLE IF NOT EXISTS cache (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL,
    expiration INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cache_expiration ON cache (expiration);

-- Jobs table (Laravel queue)
CREATE TABLE IF NOT EXISTS jobs (
    id BIGSERIAL PRIMARY KEY,
    queue VARCHAR(255) NOT NULL,
    payload TEXT NOT NULL,
    attempts SMALLINT NOT NULL,
    reserved_at INTEGER,
    available_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_jobs_queue ON jobs (queue);

-- Sessions table (Laravel sessions)
CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(255) PRIMARY KEY,
    user_id BIGINT,
    ip_address VARCHAR(45),
    user_agent TEXT,
    payload TEXT NOT NULL,
    last_activity INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_last_activity ON sessions (last_activity);

-- ============================================
-- ФУНКЦИИ И ТРИГГЕРЫ
-- ============================================

-- Функция для автоматического обновления updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Триггер для cms_blocks
DROP TRIGGER IF EXISTS trigger_cms_blocks_updated_at ON cms_blocks;
CREATE TRIGGER trigger_cms_blocks_updated_at
    BEFORE UPDATE ON cms_blocks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- ПРЕДСТАВЛЕНИЯ (VIEWS)
-- ============================================

-- Последние 10 позиций МКС
CREATE OR REPLACE VIEW v_iss_latest AS
SELECT
    id,
    fetched_at,
    payload->>'timestamp' AS timestamp_str,
    (payload->'iss_position'->>'latitude')::DOUBLE PRECISION AS latitude,
    (payload->'iss_position'->>'longitude')::DOUBLE PRECISION AS longitude
FROM iss_fetch_log
ORDER BY fetched_at DESC
LIMIT 10;

COMMENT ON VIEW v_iss_latest IS 'Последние 10 записей позиции МКС';

-- Статистика телеметрии за последний час
CREATE OR REPLACE VIEW v_telemetry_hourly_stats AS
SELECT
    DATE_TRUNC('hour', recorded_at) AS hour,
    COUNT(*) AS record_count,
    AVG(voltage) AS avg_voltage,
    MIN(voltage) AS min_voltage,
    MAX(voltage) AS max_voltage,
    AVG(temp) AS avg_temp,
    MIN(temp) AS min_temp,
    MAX(temp) AS max_temp
FROM telemetry_legacy
WHERE recorded_at > NOW() - INTERVAL '24 hours'
GROUP BY DATE_TRUNC('hour', recorded_at)
ORDER BY hour DESC;

COMMENT ON VIEW v_telemetry_hourly_stats IS 'Статистика телеметрии по часам за последние 24 часа';

-- Последние данные из каждого источника
CREATE OR REPLACE VIEW v_space_sources_latest AS
SELECT DISTINCT ON (source)
    source,
    fetched_at,
    payload
FROM space_sources
ORDER BY source, fetched_at DESC;

COMMENT ON VIEW v_space_sources_latest IS 'Последние данные из каждого источника (nasa_neo, apod, spacex)';

-- ============================================
-- СТАТИСТИКА И МОНИТОРИНГ
-- ============================================

-- Размеры таблиц
CREATE OR REPLACE VIEW v_table_sizes AS
SELECT
    schemaname AS schema,
    tablename AS table_name,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS total_size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) AS table_size,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) AS index_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

COMMENT ON VIEW v_table_sizes IS 'Размеры всех таблиц и индексов';

-- ============================================
-- ПРАВА ДОСТУПА
-- ============================================

-- Дать права cassiopeia_user на все таблицы
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO cassiopeia_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO cassiopeia_user;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO cassiopeia_user;

-- Установить права по умолчанию для новых объектов
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT ALL PRIVILEGES ON TABLES TO cassiopeia_user;

ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT ALL PRIVILEGES ON SEQUENCES TO cassiopeia_user;

-- ============================================
-- ОЧИСТКА СТАРЫХ ДАННЫХ (MAINTENANCE)
-- ============================================

-- Функция для удаления старых записей телеметрии (старше 30 дней)
CREATE OR REPLACE FUNCTION cleanup_old_telemetry()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM telemetry_legacy
    WHERE recorded_at < NOW() - INTERVAL '30 days';

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_telemetry() IS 'Удаляет записи телеметрии старше 30 дней';

-- Функция для удаления старых записей space_sources (старше 7 дней)
CREATE OR REPLACE FUNCTION cleanup_old_space_sources()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM space_sources
    WHERE fetched_at < NOW() - INTERVAL '7 days';

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_space_sources() IS 'Удаляет записи space_sources старше 7 дней';

-- ============================================
-- КОНЕЦ СХЕМЫ
-- ============================================

-- Вывод информации о созданных объектах
SELECT
    'Tables created' AS status,
    COUNT(*) AS count
FROM information_schema.tables
WHERE table_schema = 'public' AND table_type = 'BASE TABLE'

UNION ALL

SELECT
    'Indexes created' AS status,
    COUNT(*) AS count
FROM pg_indexes
WHERE schemaname = 'public'

UNION ALL

SELECT
    'Views created' AS status,
    COUNT(*) AS count
FROM information_schema.views
WHERE table_schema = 'public';
