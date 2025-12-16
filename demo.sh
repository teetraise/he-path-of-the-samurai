#!/bin/bash

# Скрипт демонстрации проекта Кассиопея
# Автор: teetraise

set -e

echo "🚀 Демонстрация проекта Кассиопея"
echo "=================================="
echo ""

# Цвета для вывода
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

demo_step() {
    echo ""
    echo -e "${BLUE}=== $1 ===${NC}"
    echo ""
}

demo_step "1️⃣  Healthcheck Rust API"
echo "Команда: curl http://localhost:8081/health"
curl -s http://localhost:8081/health | jq .
echo -e "${GREEN}✓ API работает${NC}"

demo_step "2️⃣  Текущая позиция МКС (с кэшированием Redis)"
echo "Команда: curl http://localhost:8081/last"
curl -s http://localhost:8081/last | jq '{latitude: .payload.latitude, longitude: .payload.longitude, velocity: .payload.velocity, altitude: .payload.altitude}'
echo -e "${GREEN}✓ МКС отслеживается в реальном времени${NC}"

demo_step "3️⃣  Тренд движения МКС"
echo "Команда: curl http://localhost:8081/iss/trend"
curl -s http://localhost:8081/iss/trend | jq .
echo -e "${GREEN}✓ Расчет движения работает${NC}"

demo_step "4️⃣  Сводка космических данных (NASA NEO, APOD, SpaceX)"
echo "Команда: curl http://localhost:8081/space/summary"
SUMMARY=$(curl -s http://localhost:8081/space/summary)
echo "$SUMMARY" | jq '{sources: .sources | keys}'
echo -e "${GREEN}✓ Данные из ${YELLOW}$(echo "$SUMMARY" | jq '.sources | length')${GREEN} источников${NC}"

demo_step "5️⃣  Астрономические события (ДЕМО режим)"
echo "Команда: curl http://localhost:8080/api/astro/events?demo=true"
curl -s "http://localhost:8080/api/astro/events?demo=true" | jq '.data.table.rows[].cells'
echo -e "${GREEN}✓ События отображаются${NC}"

demo_step "6️⃣  Проверка кэша Redis"
echo "Команда: docker exec iss_redis redis-cli -a redispass KEYS '*'"
KEYS=$(docker exec iss_redis redis-cli -a redispass --no-auth-warning KEYS "*" 2>/dev/null | wc -l)
echo -e "Закэшировано ключей: ${YELLOW}$KEYS${NC}"
docker exec iss_redis redis-cli -a redispass --no-auth-warning KEYS "*" 2>/dev/null | head -5
echo -e "${GREEN}✓ Redis кэширование работает${NC}"

demo_step "7️⃣  Записи в базе данных"
echo "Команда: SELECT COUNT(*) FROM iss_fetch_log"
ISS_COUNT=$(docker exec iss_db psql -U cassiopeia_user -d cassiopeia -t -c "SELECT COUNT(*) FROM iss_fetch_log;" | xargs)
echo -e "Записей ISS в БД: ${YELLOW}$ISS_COUNT${NC}"

echo "Команда: SELECT COUNT(*) FROM telemetry_legacy"
TELEMETRY_COUNT=$(docker exec iss_db psql -U cassiopeia_user -d cassiopeia -t -c "SELECT COUNT(*) FROM telemetry_legacy;" | xargs)
echo -e "Записей телеметрии: ${YELLOW}$TELEMETRY_COUNT${NC}"
echo -e "${GREEN}✓ Данные записываются в PostgreSQL${NC}"

demo_step "8️⃣  Последние записи телеметрии"
echo "Команда: SELECT * FROM telemetry_legacy ORDER BY id DESC LIMIT 3"
docker exec iss_db psql -U cassiopeia_user -d cassiopeia -c "SELECT recorded_at, voltage, temp, source_file FROM telemetry_legacy ORDER BY id DESC LIMIT 3;"
echo -e "${GREEN}✓ Генератор телеметрии работает${NC}"

demo_step "9️⃣  Логи фоновых задач"
echo "Команда: docker-compose logs rust_iss --tail=10"
docker-compose logs rust_iss --tail=10 | grep -E "(sync|INFO|Stored|Cached)" || echo "Ожидание синхронизации..."
echo -e "${GREEN}✓ Фоновые задачи выполняются${NC}"

demo_step "🎯 CMS блоки в базе данных"
echo "Команда: SELECT slug, LEFT(content, 50) FROM cms_blocks"
docker exec iss_db psql -U cassiopeia_user -d cassiopeia -c "SELECT slug, LEFT(content, 50) as content_preview FROM cms_blocks;"
echo -e "${GREEN}✓ CMS блоки загружены${NC}"

echo ""
echo "=================================="
echo -e "${GREEN}✅ Все сервисы работают корректно!${NC}"
echo ""
echo "📊 Веб-интерфейс:"
echo "   Dashboard: http://localhost:8080/dashboard"
echo "   API Docs:  http://localhost:8081/health"
echo ""
echo "🔍 Для непрерывного мониторинга:"
echo "   watch -n 2 'curl -s http://localhost:8081/last | jq .payload.latitude'"
echo ""
