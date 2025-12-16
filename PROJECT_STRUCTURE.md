# Структура проекта Кассиопея

## Обзор архитектуры

```
he-path-of-the-samurai/
├── services/
│   ├── rust-iss/          # Rust API - основной бэкенд
│   ├── php-web/           # Laravel - веб-интерфейс
│   └── telemetry-generator/ # Генератор телеметрии
├── db/                     # PostgreSQL конфигурация
├── nginx/                  # Nginx конфигурация
└── docker-compose.yml      # Оркестрация сервисов
```

## Сервисы

### 1. rust_iss (Rust API) - порт 8081

**Технологии**: Rust 1.85, Axum, SQLx, Redis, Tokio

**Архитектура** (Clean Architecture):
```
src/
├── domain/        # Бизнес-логика и модели
│   ├── iss.rs     # ISS данные (IssFetchLog, IssPosition, Trend)
│   ├── osdr.rs    # OSDR исследования NASA
│   └── space.rs   # Космические данные (NASA, SpaceX)
├── repo/          # Работа с БД (Repository Pattern)
│   ├── iss_repo.rs
│   ├── osdr_repo.rs
│   └── space_repo.rs
├── services/      # Бизнес-сервисы
│   ├── iss_service.rs    # Получение данных МКС
│   ├── osdr_service.rs   # Синхронизация OSDR
│   ├── space_service.rs  # Агрегация космических данных
│   └── scheduler.rs      # Фоновые задачи
├── handlers/      # HTTP обработчики
├── routes/        # Маршруты API
├── middleware/    # Rate limiting
└── errors/        # Унифицированная обработка ошибок
```

**API Endpoints**:
- `GET /health` - Healthcheck
- `GET /last` - Последняя позиция МКС (кэш Redis)
- `GET /fetch` - Принудительный запрос МКС
- `GET /iss/trend` - Расчет движения МКС
- `GET /osdr/sync` - Синхронизация NASA OSDR
- `GET /osdr/list` - Список исследований
- `GET /space/:src/latest` - Последние данные из источника
- `GET /space/refresh` - Обновить все источники
- `GET /space/summary` - Сводка всех источников

**Фоновые задачи** (каждые 30 сек):
- Опрос API МКС
- Синхронизация NASA OSDR
- Обновление SpaceX, NASA NEO, NASA APOD

**Кэширование**: Redis с TTL 30 секунд для позиции МКС

---

### 2. php_web (Laravel) - порт 8080

**Технологии**: PHP 8.2, Laravel 11, Blade templates

**Структура**:
```
laravel-patches/
├── app/Http/Controllers/
│   └── AstroController.php  # AstronomyAPI интеграция
├── resources/views/
│   └── dashboard.blade.php  # Главный дашборд
└── routes/web.php
```

**Функционал**:
- Dashboard с картой МКС (Leaflet.js)
- График скорости (Chart.js)
- Астрономические события (AstronomyAPI)
- CMS блоки
- Фото космоса (NASA APOD)

---

### 3. telemetry_generator (Rust) - фоновый процесс

**Функция**: Генерирует синтетическую телеметрию каждые 10 секунд

**Данные**:
- Напряжение: 11.8-12.6V
- Температура: 18-22°C
- Источник файла: случайный из списка

---

### 4. PostgreSQL (iss_db) - порт 5432

**Конфигурация**:
- Пользователь: `cassiopeia_user`
- База: `cassiopeia`
- Оптимизация для SSD (random_page_cost=1.1)
- Логирование медленных запросов (>1s)

---

### 5. Redis (iss_redis) - порт 6379

**Назначение**: Кэш для данных МКС (TTL 30 сек)

---

### 6. Nginx (web_nginx) - порт 8080

**Роль**: Reverse proxy для PHP-FPM

---

## База данных PostgreSQL

### Таблица: iss_fetch_log
Журнал запросов к API МКС

| Поле | Тип | Описание |
|------|-----|----------|
| id | BIGSERIAL PRIMARY KEY | Уникальный ID |
| fetched_at | TIMESTAMPTZ NOT NULL DEFAULT NOW() | Время запроса |
| source_url | TEXT NOT NULL | URL источника |
| payload | JSONB NOT NULL | Полный JSON ответ API |

**Индексы**:
- `idx_iss_fetch_log_fetched_at` на `fetched_at DESC`

**Назначение**: История всех запросов к API МКС для анализа движения

---

### Таблица: telemetry_legacy
Синтетическая телеметрия космических систем

| Поле | Тип | Описание |
|------|-----|----------|
| id | BIGSERIAL PRIMARY KEY | Уникальный ID |
| recorded_at | TIMESTAMPTZ NOT NULL DEFAULT NOW() | Время записи |
| voltage | DOUBLE PRECISION | Напряжение (V) |
| temp | DOUBLE PRECISION | Температура (°C) |
| source_file | TEXT | Имя исходного файла |

**Индексы**:
- `idx_telemetry_recorded_at` на `recorded_at DESC`

**Назначение**: Демонстрация работы фонового генератора данных

---

### Таблица: osdr_studies
Исследования из NASA Open Science Data Repository

| Поле | Тип | Описание |
|------|-----|----------|
| id | BIGSERIAL PRIMARY KEY | Уникальный ID |
| study_id | TEXT UNIQUE NOT NULL | ID исследования в OSDR |
| title | TEXT | Название исследования |
| description | TEXT | Описание |
| payload | JSONB | Полные метаданные |
| synced_at | TIMESTAMPTZ DEFAULT NOW() | Время синхронизации |

**Индексы**:
- `idx_osdr_studies_study_id` на `study_id`
- `idx_osdr_studies_synced_at` на `synced_at DESC`

**Назначение**: Кэширование данных исследований NASA для офлайн-доступа

---

### Таблица: space_sources
Агрегированные данные из внешних космических API

| Поле | Тип | Описание |
|------|-----|----------|
| id | BIGSERIAL PRIMARY KEY | Уникальный ID |
| source | TEXT NOT NULL | Имя источника (nasa_neo, apod, spacex) |
| fetched_at | TIMESTAMPTZ NOT NULL DEFAULT NOW() | Время получения |
| payload | JSONB NOT NULL | JSON данные |

**Индексы**:
- `idx_space_sources_source_fetched` на `source, fetched_at DESC`

**Назначение**: Хранение данных от:
- **nasa_neo**: Near-Earth Objects (астероиды)
- **apod**: Astronomy Picture of the Day
- **spacex**: Информация о запусках SpaceX

---

### Таблица: cms_blocks (Laravel)
CMS блоки для текстового контента на сайте

| Поле | Тип | Описание |
|------|-----|----------|
| id | BIGSERIAL PRIMARY KEY | Уникальный ID |
| slug | VARCHAR(255) UNIQUE NOT NULL | Уникальный идентификатор |
| content | TEXT NOT NULL | Содержимое блока |
| is_active | BOOLEAN DEFAULT TRUE | Активен ли блок |
| created_at | TIMESTAMP | Время создания |
| updated_at | TIMESTAMP | Время обновления |

**Назначение**: Управление текстовыми блоками на дашборде

---

### Таблица: users, cache, jobs, sessions (Laravel)
Стандартные таблицы Laravel для:
- Аутентификация пользователей
- Кэш приложения
- Очередь задач
- Сессии

---

## Внешние API

### 1. ISS Location API
- **URL**: `http://api.open-notify.org/iss-now.json`
- **Частота**: каждые 30 секунд
- **Данные**: широта, долгота, временная метка

### 2. NASA Open Science Data Repository (OSDR)
- **URL**: `https://osdr.nasa.gov/osdr/data/osd/meta`
- **Частота**: каждые 30 секунд
- **Данные**: исследования, эксперименты, датасеты

### 3. NASA Near-Earth Objects (NEO)
- **URL**: `https://api.nasa.gov/neo/rest/v1/feed`
- **Ключ**: NASA_API_KEY
- **Данные**: астероиды, опасные объекты

### 4. NASA APOD (Astronomy Picture of the Day)
- **URL**: `https://api.nasa.gov/planetary/apod`
- **Ключ**: NASA_API_KEY
- **Данные**: ежедневное фото космоса

### 5. SpaceX API
- **URL**: `https://api.spacexdata.com/v5/launches/latest`
- **Данные**: последний запуск SpaceX

### 6. AstronomyAPI
- **URL**: `https://api.astronomyapi.com/api/v2/bodies/events`
- **Auth**: Basic (app_id:secret)
- **Данные**: астрономические события (затмения, фазы луны)

---

## Переменные окружения

```env
# PostgreSQL
POSTGRES_USER=cassiopeia_user
POSTGRES_PASSWORD=secure_password
POSTGRES_DB=cassiopeia

# Redis
REDIS_PASSWORD=redispass

# NASA API
NASA_API_KEY=your_nasa_api_key

# AstronomyAPI
ASTRO_APP_ID=your_app_id
ASTRO_APP_SECRET=your_app_secret

# Laravel
APP_KEY=base64:...
DB_CONNECTION=pgsql
DB_HOST=db
DB_PORT=5432
DB_DATABASE=cassiopeia
DB_USERNAME=cassiopeia_user
DB_PASSWORD=secure_password

REDIS_HOST=redis
REDIS_PASSWORD=redispass
REDIS_PORT=6379
```

---

## Запуск проекта

### Первый запуск:
```bash
# 1. Настроить .env файлы
cp .env.example .env
# Заполнить NASA_API_KEY, ASTRO_APP_ID, ASTRO_APP_SECRET

# 2. Запустить все сервисы
docker-compose up -d

# 3. Подождать 30 секунд для инициализации БД
sleep 30

# 4. Запустить миграции Laravel
docker exec php_web php artisan migrate --force

# 5. Перезапустить Rust API
docker-compose restart rust_iss
```

### Демонстрация:
```bash
chmod +x demo.sh
./demo.sh
```

---

## Мониторинг

### Проверка здоровья сервисов:
```bash
docker-compose ps
curl http://localhost:8081/health
```

### Логи:
```bash
docker-compose logs -f rust_iss
docker-compose logs -f telemetry_generator
```

### Redis:
```bash
docker exec iss_redis redis-cli -a redispass KEYS "*"
docker exec iss_redis redis-cli -a redispass TTL "iss:last"
```

### PostgreSQL:
```bash
docker exec iss_db psql -U cassiopeia_user -d cassiopeia
```

---

## Производительность

- **Rate Limiting**: 100 запросов/минуту на Rust API
- **Redis TTL**: 30 секунд для кэша МКС
- **Background Tasks**: Каждые 30 секунд (ISS, OSDR, Space APIs)
- **Telemetry Generator**: Каждые 10 секунд

---

## Архитектурные решения

### 1. Почему Rust для API?
- Высокая производительность
- Безопасность типов
- Асинхронность (Tokio)
- Низкое потребление памяти

### 2. Почему Laravel для фронтенда?
- Быстрая разработка UI
- Blade templates
- Удобная работа с сессиями
- Интеграция с PostgreSQL

### 3. Почему Redis?
- Кэширование частых запросов к ISS API
- TTL для автоматической инвалидации
- Снижение нагрузки на внешние API

### 4. Почему PostgreSQL?
- JSONB для гибкого хранения API ответов
- Мощные индексы на JSONB
- Надежность и ACID
- GIN индексы для JSON запросов

### 5. Clean Architecture
- Разделение на слои (domain, repo, services, handlers)
- Легкое тестирование
- Независимость от фреймворков
- Гибкость при изменениях

---

## TODO / Возможные улучшения

- [ ] WebSocket для real-time обновлений МКС
- [ ] Prometheus метрики
- [ ] Grafana дашборд
- [ ] Unit тесты (Rust)
- [ ] Integration тесты (Docker)
- [ ] CI/CD pipeline
- [ ] Kubernetes манифесты
- [ ] GraphQL API
- [ ] User authentication
- [ ] API rate limiting per user
