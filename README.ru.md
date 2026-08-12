# opencode-proxy-rs

[![CI](https://github.com/xodapi/opencode-proxy/actions/workflows/ci.yml/badge.svg)](https://github.com/xodapi/opencode-proxy/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 1.70+](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

Высокопроизводительный прокси для OpenCode Zen API, совместимый с OpenAI. Один 2.6 MB бинарик без внешних зависимостей.

**Ключевые особенности:**
- ⚡ 50x меньше, чем Node.js версия (2.6 MB vs 50 MB)
- 🚀 Асинхронные запросы с Tokio
- 📊 Встроенный дашборд и метрики
- 🔄 Потоковая передача SSE
- 🎯 Балансировка (round-robin, random)
- 💾 Аналитика использования
- 🔐 Аутентификация по токену
- 🛡️ Security headers & CSP
- ✅ 17+ тестов, 100% pass rate

## Быстрый старт

### Установка

**Вариант 1: Скачать бинарник (Windows)**
```powershell
# Из릴리زов
wget https://github.com/xodapi/opencode-proxy/releases/download/v1.7.0/opencode-proxy.exe
.\opencode-proxy.exe
```

**Вариант 2: Собрать из исходников**
```bash
git clone https://github.com/xodapi/opencode-proxy.git opencode-proxy-rs
cd opencode-proxy-rs
cargo build --release
./target/release/opencode-proxy
```

### Запуск

```bash
# По умолчанию: http://127.0.0.1:3000
./opencode-proxy

# Свой порт
PORT=3001 ./opencode-proxy

# Свой upstream
UPSTREAM_URL=https://api.example.com/v1 ./opencode-proxy

# С debug-логами
RUST_LOG=debug ./opencode-proxy
```

## Конфигурация

Всё через переменные окружения (без конфиг-файла):

| Переменная | По умолчанию | Описание |
|-----------|-------------|----------|
| `HOST` | `127.0.0.1` | Адрес для bind |
| `PORT` | `3000` | Порт |
| `MODELS` | 5 моделей | Доступные модели (через запятую) |
| `ROUTING` | `round-robin` | Стратегия: `round-robin` или `random` |
| `UPSTREAM_URL` | `https://opencode.ai/zen/v1` | Upstream API |
| `UPSTREAM_TIMEOUT` | `30` | Timeout в секундах |
| `MANAGEMENT_TOKEN` | (пусто) | Токен для /dashboard, /metrics |
| `USAGE_DB_PATH` | `~/.config/opencode-proxy/usage.jsonl` | Путь к БД аналитики |
| `USAGE_RETENTION_DAYS` | `30` | Дней хранить историю |

### Пример .env файла

```bash
HOST=127.0.0.1
PORT=3000
MODELS=gpt-4,gpt-4-turbo,gpt-3.5-turbo
MANAGEMENT_TOKEN=my-secret-token-123
ROUTING=round-robin
```

## API Эндпоинты

### Открытые (без аутентификации)

```bash
# Проверка здоровья
curl http://127.0.0.1:3000/health
# → {"status":"ok"}

# Список моделей (совместимо с OpenAI)
curl http://127.0.0.1:3000/v1/models

# Chat completions (совместимо с OpenAI)
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4","messages":[{"role":"user","content":"Привет"}]}'
```

### Защищённые (требуют MANAGEMENT_TOKEN)

```bash
# Дашборд (веб-интерфейс)
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://127.0.0.1:3000/dashboard

# JSON метрики
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://127.0.0.1:3000/metrics

# Статистика использования
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://127.0.0.1:3000/usage

# Экспорт (CSV/JSON)
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://127.0.0.1:3000/export/csv
```

## Примеры использования

### Python клиент

```python
import openai

openai.api_base = "http://127.0.0.1:3000/v1"
openai.api_key = "dummy-key"  # не используется прокси

response = openai.ChatCompletion.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Объясни асинхронность в Rust"}]
)
print(response.choices[0].message.content)
```

### JavaScript клиент

```javascript
const response = await fetch('http://127.0.0.1:3000/v1/chat/completions', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    model: 'gpt-4',
    messages: [{ role: 'user', content: 'Привет!' }]
  })
});

const data = await response.json();
console.log(data.choices[0].message.content);
```

### Потоковая передача

```bash
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model":"gpt-4",
    "messages":[{"role":"user","content":"Напиши стихотворение"}],
    "stream":true
  }' | grep "data:" | sed 's/data: //' | jq '.choices[0].delta.content'
```

## Архитектура

```
opencode-proxy-rs/
├── src/
│   ├── main.rs              # Точка входа
│   ├── server.rs            # HTTP маршруты
│   ├── proxy.rs             # Проксирование (JSON + SSE)
│   ├── router.rs            # Балансировка (round-robin, random)
│   ├── config.rs            # Парсинг ENV
│   ├── auth.rs              # Валидация токена
│   ├── circuit_breaker.rs   # Обработка ошибок
│   ├── usage_store.rs       # JSONL хранилище
│   ├── models.rs            # Структуры данных (serde)
│   ├── metrics/             # Сбор метрик
│   ├── templates/           # HTML/CSS дашборд
│   └── export.rs            # CSV/JSON экспорт
├── tests/
│   └── integration.rs       # Интеграционные тесты
└── Cargo.toml
```

### Ключевые решения

**Один бинарик**: Все зависимости статически слинкованы. Нет npm install, нет Docker-слоёв.

**Tokio для асинхронности**: Многопоточный runtime с work-stealing scheduler.

**Ring buffer для метрик**: Фиксированный размер окна (1 час), старые данные автоматически удаляются.

**JSONL для аналитики**: Append-only логирование, быстрые записи, минимум аллокаций.

**CSP + Security headers**: Дашборд защищён современными политиками браузера.

## Тестирование

### Запуск всех тестов

```bash
cargo test
```

### Запуск с выводом

```bash
cargo test -- --nocapture
```

### Конкретный тест

```bash
cargo test test_round_robin
```

### Интеграционные тесты

```bash
cargo test --test integration
```

### Проверка качества кода

```bash
# Форматирование
cargo fmt --check

# Linting
cargo clippy -- -D warnings

# Оба вместе
cargo fmt && cargo clippy -- -D warnings
```

## Производительность

| Метрика | Значение |
|---------|---------|
| Размер бинарика | 2.6 MB |
| Время запуска | ~100 ms |
| Задержка запроса | <1 ms (без upstream) |
| Память (холостой ход) | ~15 MB |
| Пропускная способность | ~1000 req/s (один core) |

**Сравнение с Node.js версией:**

| Аспект | Rust | Node.js |
|--------|------|---------|
| Размер | 2.6 MB | 50 MB |
| Запуск | 100 ms | 500 ms |
| Память | 15 MB | 45 MB |
| Зависимости | 0 | 42 npm пакета |

## Развёртывание

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/opencode-proxy /opencode-proxy
EXPOSE 3000
CMD ["/opencode-proxy"]
```

```bash
docker build -t opencode-proxy .
docker run -p 3000:3000 \
  -e MANAGEMENT_TOKEN=secret \
  opencode-proxy
```

### Systemd (Linux)

```ini
[Unit]
Description=OpenCode Proxy
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/opencode-proxy
Restart=on-failure
Environment="PORT=3000"
Environment="MANAGEMENT_TOKEN=secret"

[Install]
WantedBy=multi-user.target
```

### Nginx (reverse proxy)

```nginx
upstream opencode {
    server 127.0.0.1:3000;
}

server {
    listen 443 ssl http2;
    server_name api.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://opencode;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

## Безопасность

- ✅ Промпты и ответы не хранятся
- ✅ API-ключи только в памяти
- ✅ MANAGEMENT_TOKEN для защиты эндпоинтов
- ✅ CSP headers на дашборде
- ✅ X-Content-Type-Options, X-Frame-Options установлены
- ⚠️ HTTPS не обязателен (использовать reverse proxy в production)

Подробнее см. [SECURITY.md](SECURITY.md).

## Лицензия

MIT License — см. [LICENSE](LICENSE)

## Контрибьютинг

Приветствуются PR! Читайте [CONTRIBUTING.md](CONTRIBUTING.md) для guidelines.

## Changelog

История версий в [CHANGELOG.md](CHANGELOG.md).

## Благодарности

- [Axum](https://github.com/tokio-rs/axum) — HTTP framework
- [Tokio](https://tokio.rs) — async runtime
- [Serde](https://serde.rs) — serialization
- OpenCode team за Zen API
