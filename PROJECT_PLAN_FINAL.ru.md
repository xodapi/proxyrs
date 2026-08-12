# opencode-proxy-rs — Финальный план проекта

**Статус:** Rust-реализация OpenCode Zen API proxy. Phase 0-6 завершены.
JS-прокси (исходный) продолжает работать на `:3000` без изменений.
Rust-версия в `C:\project\opencode-rs` компилируется, проходит тесты,
готова к параллельному запуску на `:3001`.

**Дата фиксации:** 2 июля 2026.

---

## 0. Что уже сделано (не трогать, не пересматривать)

### Rust-прокси (C:\project\opencode-rs)

| Фаза | Компонент | Статус |
|------|-----------|--------|
| 0 | Scaffold: axum + tokio + сборка | ✅ |
| 0 | Config из ENV (HOST, PORT, MODELS, ROUTING...) | ✅ |
| 0 | Auth: MANAGEMENT_TOKEN защита | ✅ |
| 0 | Router: round-robin / random | ✅ |
| 0 | Тесты: 17 unit-тестов | ✅ |
| 0 | Release build: 2.6 MB, opt-level="z" | ✅ |
| 1 | POST /v1/chat/completions — upstream passthrough | ✅ |
| 1 | GET /v1/models — список моделей | ✅ |
| 1 | Metrics recording (ring buffer) | ✅ |
| 2 | Streaming SSE passthrough (mpsc канал) | ✅ |
| 2 | Парсинг usage из SSE + JSON | ✅ |
| 2 | Rate-limit header extraction | ✅ |
| 3 | UsageStore: JSONL-файл на диск | ✅ |
| 3 | Агрегация по дням/моделям | ✅ |
| 3 | Prune старых событий | ✅ |
| 4 | CSP-заголовки (dashboard + flow) | ✅ |
| 4 | X-Content-Type-Options / X-Frame-Options | ✅ |
| 5 | build-release.ps1 — скрипт сборки | ✅ |
| 5 | Docker/self-update — пока не нужно | ❌ |

### JS-прокси (C:\project\opencode — исходный, работает на :3000)

- Полный дашборд с Chart.js
- Flow-страница с vanilla JS
- /diag эндпоинт
- 83 теста проходят
- AGENTS.md зафиксирован

---

## 1. Что можно улучшить БЕЗ переключения (не ломает JS)

### 1.1 Рефакторинг JS-прокси (безопасно, файлы на диске)

```
Приоритет: косметика и документация, не требует рестарта
- AGENTS.md — добавить секцию Rust-версии
- README.md — упомянуть Rust-альтернативу
- .gitignore — добавить target/, *.lock для Rust
```

### 1.2 Rust-прокси (отдельная директория, ничего не ломает)

```
Приоритет: всё что угодно, Rust не запущен
- SECURITY.md для Rust-репозитория
- README.md для Rust (инструкция по запуску)
- GitHub Actions CI (.github/workflows/ci.yml)
- clap CLI аргументы (опционально)
- Интеграционные тесты (tests/integration.rs)
```

---

## 2. Дорожная карта Rust-версии (порядок реализации)

### Фаза A: Стабилизация (сейчас — до переключения)

```
A.1 README.md — инструкция, переменные окружения, примеры
A.2 SECURITY.md — политика безопасности
A.3 GitHub Actions CI — сборка + тесты под Windows
A.4 Интеграционные тесты — поднятие сервера, проверка эндпоинтов
A.5 Переключение: Rust на :3000, JS на :3001 (валидация)
```

### Фаза B: Функциональность (после переключения)

```
B.1 Streaming SSE — улучшенный парсинг usage (текущий: упрощённый)
B.2 Circuit breaker — отключение падающих моделей
B.3 Retry с exponential backoff — повтор при 429/5xx
B.4 UsageStore: запись pending при старте (сейчас: только новые события)
```

### Фаза C: Экосистема (через 1-2 недели стабильной работы)

```
C.1 Self-update (аналог vimit) — проверка новой версии на GitHub
C.2 Подпись бинарника — Authenticode на Windows
C.3 Linux/macOS сборки — GitHub Actions matrix
C.4 WebSocket для real-time flow
```

---

## 3. Порядок реализации (фаза A, детально)

### A.1 README.md

```markdown
# opencode-proxy-rs

Rust-реализация OpenCode Zen API proxy. Замена Node.js-версии
с сохранением полной совместимости API.

## Быстрый старт

cargo run --release

## Переменные окружения

HOST=127.0.0.1 PORT=3001 ROUTING=round-robin cargo run --release
```

### A.2 SECURITY.md

Политика: не хранить prompts/responses/API keys в логах,
metrics/usage/dashboard — только с MANAGEMENT_TOKEN.

### A.3 GitHub Actions CI

```
name: CI
on: [push, pull_request]
jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release
      - run: cargo test --release
```

### A.4 Интеграционные тесты

```rust
// tests/integration.rs
// 1. Запуск сервера на тестовом порту
// 2. GET /health → 200
// 3. GET /v1/models → 200 + модели
// 4. POST /v1/chat/completions → 200 + usage
// 5. GET /metrics → version: 1
// 6. GET /diag → health: ok
```

### A.5 Переключение

```
Шаг 1: Остановить JS на :3000
Шаг 2: Запустить Rust на :3000 с USAGE_DB_PATH на тот же файл
Шаг 3: Проверить OpenCode Desktop → работает через Rust
Шаг 4: Если проблемы → остановить Rust, запустить JS
Шаг 5: Время роллбэка: < 10 секунд
```

---

## 4. Что НЕ входит в текущий план (явно отложено)

- **Docker-образ** — не нужен для локального Windows-first использования
- **Self-update** — когда появится регулярный релизный цикл
- **WebSocket для flow** — после стабилизации REST API
- **Anthropic `/v1/messages`** — Issue P2, не критично
- **Прометеус `/metrics/prometheus`** — Issue P3
- **Кэш запросов** — Issue P3, semantic cache research

---

## 5. Критерий готовности к переключению

```powershell
# Все проверки проходят:
cargo test --release                    # 17 тестов
cargo clippy -- -D warnings             # чистый clippy
scripts\build-release.ps1               # zip архив создаётся
.\target\release\opencode-proxy.exe     # запуск без ошибок

# Сравнение с JS на тестовых данных:
curl -s :3000/metrics | python -c "import sys,json; d=json.load(sys.stdin); print(d['version'])"
curl -s :3001/metrics | python -c "import sys,json; d=json.load(sys.stdin); print(d['version'])"
# Оба возвращают version: 1
```

---

## 6. Улучшения БЕЗ переключения (можно делать сейчас)

| Улучшение | Где | Риск |
|-----------|-----|------|
| README.md для Rust | `opencode-rs/` | Нулевой |
| SECURITY.md для Rust | `opencode-rs/` | Нулевой |
| .github/workflows/ci.yml | `opencode-rs/` | Нулевой |
| Интеграционные тесты | `opencode-rs/tests/` | Нулевой |
| AGENTS.md — добавить Rust секцию | `opencode/AGENTS.md` | Нулевой (файл) |
| .gitignore — target/ | `opencode-rs/` | Нулевой |
| ISSUES.md — отметить завершённое | `opencode/` | Нулевой (файл) |
| dashboard.js — мелкие исправления | `opencode/src/` | **Нужен рестарт JS** |

---

## 7. Итоговое резюме

**JS-прокси** (C:\project\opencode): работает на :3000, перезапускать не нужно.  
**Rust-прокси** (C:\project\opencode-rs): готов к запуску на :3001.  

**Когда будете у ноутбука:**
```powershell
# Консоль 1 — Rust
cd C:\project\opencode-rs
$env:PORT = "3001"; .\target\release\opencode-proxy.exe

# Консоль 2 — тест
curl.exe http://127.0.0.1:3001/health  # → {"status":"ok"}
```

Без риска, без остановки JS, полная совместимость.
