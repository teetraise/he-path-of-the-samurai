# Архитектура проекта "Кассиопея"

## Обзор

Кассиопея - микросервисная система сбора и обработки космических данных из открытых API.

## Сервисы

### 1. `rust_iss` - Rust API Server
**Стек:** Rust, Axum, SQLx, Redis
**Порт:** 8081 → 3000

**Архитектура:**
```
src/
├── config/          # Конфигурация из env
├── domain/          # Модели данных (ISS, OSDR, Space)
├── repo/            # Слой доступа к БД (prepared statements)
├── clients/         # HTTP клиенты с retry логикой
├── services/        # Бизнес-логика + кэширование
├── handlers/        # HTTP handlers
├── routes/          # Роутинг
├── middleware/      # Rate limiting, error handling
└── errors/          # Унифицированные ошибки
```

**Паттерны:**
- Repository Pattern - изоляция SQL
- Service Layer - бизнес-логика
- Cache-Aside (Redis) - TTL 60-3600 сек
- Retry с Exponential Backoff (max 3 попытки)
- PostgreSQL Advisory Lock - защита от наложения фоновых задач
- Rate Limiting - 100 req/min, скользящее окно

**API Endpoints:**
- GET `/health` - проверка здоровья
- GET `/last` - последняя позиция ISS
- GET `/fetch` - принудительный опрос ISS
- GET `/iss/trend` - расчет движения ISS
- GET `/osdr/sync` - синхронизация NASA OSDR
- GET `/osdr/list` - список датасетов
- GET `/space/:src/latest` - последние данные из кэша
- GET `/space/refresh` - обновление кэша
- GET `/space/summary` - сводка по всем источникам

**Формат ошибок:**
```json
{
  "ok": false,
  "error": {
    "code": "EXTERNAL_API_ERROR",
    "message": "...",
    "trace_id": "uuid"
  }
}
```
HTTP статус всегда 200 для предсказуемости.

### 2. `telemetry_generator` - Генератор телеметрии
**Стек:** Rust, SQLx
**Замена:** Pascal legacy (2008)

**Функционал:**
- Генерация CSV с телеметрией каждые N секунд
- Запись в БД (`telemetry_legacy` таблица)
- Формат CSV: `recorded_at,voltage,temp,source_file`

**Контракт сохранен:**
- Те же переменные окружения
- Тот же формат CSV
- Та же таблица в БД

### 3. `php_web` - Laravel Frontend
**Стек:** Laravel, Bootstrap, Blade
**Порт:** 8080

**Страницы:**
- `/dashboard` - главная панель
- `/osdr` - NASA OSDR данные
- `/api/iss/last` - проксирование к Rust
- `/api/iss/trend` - проксирование к Rust
- `/api/jwst/feed` - JWST галерея
- `/api/astro/events` - астрономические события

### 4. `redis` - Кэш
**Стек:** Redis 7
**Порт:** 6379

**Использование:**
- Кэширование ответов внешних API
- TTL зависит от типа данных (60-3600 сек)
- Защита паролем через `REDIS_PASSWORD`

### 5. `db` - PostgreSQL
**Стек:** PostgreSQL 16
**Порт:** 5432

**Таблицы:**
- `iss_fetch_log` - история позиций ISS
- `osdr_items` - NASA OSDR датасеты (UPSERT по `dataset_id`)
- `space_cache` - унифицированный кэш (APOD, NEO, DONKI, SpaceX)
- `telemetry_legacy` - данные телеметрии
- `cms_pages` - CMS контент

**Индексы:**
- `idx_iss_fetched_at` - на timestamp DESC
- `idx_iss_payload_gin` - GIN на JSONB
- `idx_osdr_inserted_at`, `idx_osdr_status`
- `idx_space_cache_source_time` - composite index
- `idx_telemetry_recorded_at`

**Оптимизация:**
- `shared_buffers = 256MB`
- `effective_cache_size = 1GB`
- `work_mem = 16MB`
- `random_page_cost = 1.1` (для SSD)
- Autovacuum включен для JSONB таблиц

## Безопасность

### Реализовано:
- ✅ Все секреты в `.env` (нет хардкода)
- ✅ Prepared statements (защита от SQL injection)
- ✅ TIMESTAMPTZ для всех дат (защита от timezone issues)
- ✅ Валидация через `validator` crate (Rust)
- ✅ Rate limiting (100 req/min)
- ✅ Redis с паролем
- ✅ PostgreSQL с отдельным пользователем

### Laravel:
- CSRF tokens
- XSS escape (Blade автоматически)
- Eager Loading (избежание N+1)

## Внешние API

| API | URL | Auth | Интервал |
|-----|-----|------|----------|
| ISS Position | wheretheiss.at | - | 120 сек |
| NASA OSDR | osdr.nasa.gov | API Key | 600 сек |
| APOD | api.nasa.gov/planetary/apod | API Key | 12ч |
| NeoWs | api.nasa.gov/neo | API Key | 2ч |
| DONKI | api.nasa.gov/DONKI | API Key | 1ч |
| SpaceX | api.spacexdata.com | - | 1ч |
| JWST | api.jwstapi.com | API Key | on demand |
| Astronomy | astronomyapi.com | Basic Auth | on demand |

**Retry логика:**
- Max 3 попытки
- Delays: 1s, 2s, 4s (exponential backoff)
- Timeout: 30 сек
- User-Agent: "Cassiopeia-ISS/1.0"

## Развертывание

```bash
# 1. Клонировать
git clone https://github.com/teetraise/he-path-of-the-samurai.git
cd he-path-of-the-samurai

# 2. Скопировать .env
cp .env.example .env
# Отредактировать .env с реальными ключами API

# 3. Запустить
docker-compose up --build

# 4. Проверить
curl http://localhost:8081/health
curl http://localhost:8080/dashboard
```

## Мониторинг

**Логи:**
- PostgreSQL: медленные запросы >1s в `/var/lib/postgresql/data/log/`
- Rust: через `tracing`, уровень `RUST_LOG=info`
- Redis: stdout

**Healthchecks:**
- PostgreSQL: `pg_isready` каждые 10s
- Redis: `redis-cli ping` каждые 10s
- Rust: HTTP `/health` endpoint

## Производительность

**Кэширование:**
- Redis Cache-Aside паттерн
- Hit rate ожидается >70%

**База данных:**
- Connection pooling (10 соединений для Rust API)
- Индексы на все частые запросы
- GIN индексы для JSONB поиска

**Rust:**
- Multi-stage build (итоговый образ ~100MB)
- Async I/O (Tokio)
- Connection pooling для Redis и PostgreSQL

## Изменения от монолита

### Было:
- Монолитный main.rs (545 строк)
- Нет кэширования
- Нет rate limiting
- Хардкод секретов в docker-compose
- Pascal legacy 2008 года
- Нет индексов в БД

### Стало:
- Слоистая архитектура (2158 строк, 30 модулей)
- Redis кэширование с TTL
- Rate limiting middleware
- Все секреты в .env
- Rust CLI generator
- Оптимизированная БД с индексами и настройками
- Multi-stage Docker builds
- PostgreSQL Advisory Locks
- Retry с exponential backoff
- Унифицированный формат ошибок
 
