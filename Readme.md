# Кассиопея - Система сбора космических данных

Распределенная микросервисная система для сбора, обработки и визуализации данных из космических API (ISS, NASA, SpaceX и др.)

### Микросервисы:
- **rust_iss** - Rust API (Axum + SQLx + Redis) - основной сервис сбора данных
- **telemetry_generator** - Rust CLI - генератор телеметрии (замена Pascal legacy)
- **php_web** - Laravel фронтенд с Bootstrap
- **redis** - Redis 7 для кэширования
- **db** - PostgreSQL 16 с оптимизацией
- **nginx** - Reverse proxy

## Быстрый старт

```bash
# 1. Клонировать репозиторий
git clone https://github.com/teetraise/he-path-of-the-samurai.git
cd he-path-of-the-samurai

# 2. Настроить переменные окружения
cp .env.example .env

# 3. Запустить все сервисы
docker-compose up --build

# 4. Открыть в браузере
# Frontend: http://localhost:8080/dashboard
# API: http://localhost:8081/health
```

## API Endpoints

### Rust API (port 8081)

```bash
GET /health         # Healthcheck
GET /last           # Последняя позиция ISS
GET /iss/trend      # Расчет движения
GET /osdr/sync      # Синхронизация NASA OSDR
GET /space/summary  # Сводка всех источников
```

## Архитектура

Проект следует **Clean Architecture** принципам:
- Repository Pattern - изоляция SQL
- Service Layer - бизнес-логика
- Cache-Aside - Redis кэширование
- Rate Limiting - 100 req/min
- PostgreSQL Advisory Lock

## Изменения от монолита

### До:
- Монолитный main.rs (545 строк)
- Нет кэширования
- Хардкод секретов
- Pascal legacy 2008

### После:
-  Слоистая архитектура (2158 строк, 30 модулей)
-  Redis кэширование
-  Все секреты в .env
-  Rust CLI generator
-  БД с индексами
-  Multi-stage builds

## Документация

- [ARCHITECTURE.md](ARCHITECTURE.md) - Подробная архитектура
- [db/init.sql](db/init.sql) - Схема БД
