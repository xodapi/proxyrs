# Проверка работоспособности и варианты использования

## 1️⃣ Проверить что работает локально (5 минут)

### Вариант A: Быстрая проверка (3 минуты)

```bash
# 1. Перейти в проект
cd C:\project\opencode-rs

# 2. Собрать релиз
cargo build --release

# 3. Запустить
.\target\release\opencode-proxy.exe

# В другом терминале:

# 4. Проверить здоровье
curl http://127.0.0.1:3000/health
# Ожидать: {"status":"ok"}

# 5. Проверить модели
curl http://127.0.0.1:3000/v1/models | jq '.data | length'
# Ожидать: 5 (или количество моделей)

# 6. Остановить
# Нажать Ctrl+C в первом терминале
```

### Вариант B: Полная проверка (5 минут)

```bash
# 1. Запустить тесты
cargo test --release

# 2. Проверить качество кода
cargo fmt --check
cargo clippy -- -D warnings

# 3. Собрать релиз
cargo build --release

# 4. Проверить размер бинарика
ls -lh target\release\opencode-proxy.exe
# Ожидать: ~2.6 MB

# 5. Запустить и протестировать
.\target\release\opencode-proxy.exe &

# Подождать 2 секунды
Start-Sleep -Seconds 2

# 6. Тест здоровья
curl http://127.0.0.1:3000/health

# 7. Тест моделей
curl http://127.0.0.1:3000/v1/models

# 8. Остановить
Stop-Process -Name opencode-proxy
```

---

## 2️⃣ Варианты использования на сервере

### Вариант 1: Windows Service (самый просто)

**Шаг 1: Установить NSSM (Windows Service Manager)**
```powershell
choco install nssm
```

**Шаг 2: Установить как сервис**
```powershell
# Путь к exe
$exePath = "C:\opencode-proxy\opencode-proxy.exe"

# Создать сервис
nssm install OpenCodeProxy "$exePath"

# Установить переменные окружения
nssm set OpenCodeProxy AppEnvironmentExtra PORT=3000
nssm set OpenCodeProxy AppEnvironmentExtra MANAGEMENT_TOKEN=your-secret-token-123
nssm set OpenCodeProxy AppEnvironmentExtra UPSTREAM_URL=https://opencode.ai/zen/v1

# Запустить сервис
nssm start OpenCodeProxy

# Проверить статус
nssm status OpenCodeProxy
```

**Проверить:**
```powershell
# Должен быть запущен
curl http://127.0.0.1:3000/health

# Остановить/перезапустить
nssm stop OpenCodeProxy
nssm start OpenCodeProxy
```

---

### Вариант 2: Docker (если на Linux сервере)

**Шаг 1: Собрать Docker образ**
```bash
cd /path/to/opencode-proxy
docker build -t opencode-proxy:1.7.0 .
```

**Шаг 2: Запустить контейнер**
```bash
docker run -d \
  --name opencode-proxy \
  -p 3000:3000 \
  -e MANAGEMENT_TOKEN=secret-token-123 \
  -e UPSTREAM_URL=https://opencode.ai/zen/v1 \
  -v /var/lib/opencode-proxy:/root/.config/opencode-proxy \
  opencode-proxy:1.7.0
```

**Проверить:**
```bash
# Должен быть запущен
docker logs opencode-proxy

# Тест
curl http://localhost:3000/health

# Остановить/перезапустить
docker stop opencode-proxy
docker start opencode-proxy
```

---

### Вариант 3: Systemd (Linux)

**Шаг 1: Скопировать бинарик**
```bash
sudo cp target/release/opencode-proxy /usr/local/bin/
sudo chmod +x /usr/local/bin/opencode-proxy
```

**Шаг 2: Создать systemd сервис**
```bash
sudo tee /etc/systemd/system/opencode-proxy.service > /dev/null <<EOF
[Unit]
Description=OpenCode Proxy
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/opencode-proxy
Restart=on-failure
RestartSec=5
User=nobody
Environment="PORT=3000"
Environment="MANAGEMENT_TOKEN=your-secret-token"
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF
```

**Шаг 3: Включить и запустить**
```bash
sudo systemctl daemon-reload
sudo systemctl enable opencode-proxy
sudo systemctl start opencode-proxy

# Проверить статус
sudo systemctl status opencode-proxy

# Логи
sudo journalctl -u opencode-proxy -f
```

**Проверить:**
```bash
curl http://localhost:3000/health
```

---

### Вариант 4: Nginx Reverse Proxy (для HTTPS)

**Шаг 1: Конфиг Nginx**
```nginx
# /etc/nginx/sites-available/opencode-proxy

upstream opencode_backend {
    server 127.0.0.1:3000;
}

server {
    listen 443 ssl http2;
    server_name api.yourdomain.com;

    # Сертификаты
    ssl_certificate /etc/letsencrypt/live/api.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.yourdomain.com/privkey.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000" always;
    add_header X-Content-Type-Options "nosniff" always;

    location / {
        proxy_pass http://opencode_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name api.yourdomain.com;
    return 301 https://$server_name$request_uri;
}
```

**Шаг 2: Включить сайт**
```bash
sudo ln -s /etc/nginx/sites-available/opencode-proxy \
           /etc/nginx/sites-enabled/

sudo nginx -t  # проверить синтаксис

sudo systemctl restart nginx
```

**Проверить:**
```bash
# Через HTTPS
curl https://api.yourdomain.com/health
```

---

### Вариант 5: Systemd + Nginx + Auto-restart

**Создать упраощённый стек:**

```bash
# 1. Systemd сервис (как выше)
sudo systemctl start opencode-proxy

# 2. Nginx конфиг (как выше)
sudo systemctl restart nginx

# 3. Healthcheck скрипт (автоперезапуск)
# /usr/local/bin/healthcheck-opencode.sh

#!/bin/bash
if ! curl -f http://127.0.0.1:3000/health > /dev/null 2>&1; then
    echo "OpenCode Proxy down, restarting..."
    sudo systemctl restart opencode-proxy
    sleep 2
fi

# 4. Добавить в crontab
crontab -e

# Добавить строку:
*/5 * * * * /usr/local/bin/healthcheck-opencode.sh
```

---

## 3️⃣ Полный checklist для продакшена

```bash
☐ Собрать релиз: cargo build --release
☐ Проверить тесты: cargo test
☐ Проверить качество: cargo fmt && cargo clippy -- -D warnings
☐ Проверить размер: ls -lh target/release/opencode-proxy
☐ Запустить локально: ./target/release/opencode-proxy
☐ Протестировать endpoints:
  ☐ curl http://127.0.0.1:3000/health
  ☐ curl http://127.0.0.1:3000/v1/models
  ☐ curl -H "Authorization: Bearer TOKEN" http://127.0.0.1:3000/metrics
☐ Проверить конфиг (.env файл):
  ☐ HOST, PORT
  ☐ UPSTREAM_URL (проверить доступность)
  ☐ MANAGEMENT_TOKEN (установить сильный токен)
  ☐ MODELS (список моделей)
☐ Выбрать вариант развёртывания (Windows Service / Docker / Systemd)
☐ Установить сервис на сервер
☐ Проверить здоровье: curl https://api.yourdomain.com/health
☐ Настроить мониторинг (healthcheck каждые 5 минут)
☐ Сделать бэкап конфига
```

---

## 4️⃣ Быстрый старт на своём сервере (syntog.ru пример)

```bash
# На сервере:

# 1. SSH на сервер
ssh user@syntog.ru

# 2. Скачать релиз или собрать
git clone https://github.com/xodapi/opencode-proxy.git
cd opencode-proxy
cargo build --release

# 3. Скопировать exe
sudo cp target/release/opencode-proxy /usr/local/bin/

# 4. Создать .env
sudo tee /etc/default/opencode-proxy > /dev/null <<EOF
PORT=3000
MANAGEMENT_TOKEN=your-secret-token-here
UPSTREAM_URL=https://opencode.ai/zen/v1
USAGE_DB_PATH=/var/lib/opencode-proxy/usage.jsonl
EOF

# 5. Создать директорию для данных
sudo mkdir -p /var/lib/opencode-proxy
sudo chown nobody:nogroup /var/lib/opencode-proxy

# 6. Создать systemd сервис (см. Вариант 3 выше)

# 7. Запустить
sudo systemctl start opencode-proxy
sudo systemctl status opencode-proxy

# 8. Добавить Nginx reverse proxy (см. Вариант 4 выше)

# 9. Проверить через HTTPS
curl https://api.yourdomain.com/health
```

---

## 5️⃣ Мониторинг на сервере

**Скрипт проверки здоровья:**

```bash
#!/bin/bash
# /usr/local/bin/monitor-opencode.sh

ENDPOINT="http://127.0.0.1:3000/health"
RESPONSE=$(curl -s -w "%{http_code}" "$ENDPOINT" -o /dev/null)

if [ "$RESPONSE" != "200" ]; then
    echo "❌ OpenCode Proxy down (HTTP $RESPONSE)"
    systemctl restart opencode-proxy
    sleep 2
    RESPONSE=$(curl -s -w "%{http_code}" "$ENDPOINT" -o /dev/null)
    if [ "$RESPONSE" = "200" ]; then
        echo "✅ Restarted successfully"
    else
        echo "❌ Still down after restart!"
        # Отправить уведомление
        echo "OpenCode Proxy failed" | mail -s "Alert" admin@yourdomain.com
    fi
else
    echo "✅ OpenCode Proxy healthy"
fi
```

**Добавить в crontab (проверка каждые 5 минут):**
```bash
*/5 * * * * /usr/local/bin/monitor-opencode.sh >> /var/log/opencode-proxy-monitor.log 2>&1
```

---

## ❓ Какой вариант выбрать?

| Вариант | Платформа | Сложность | Рекомендуется для |
|---------|-----------|----------|------------------|
| **Windows Service** | Windows | ⭐⭐ | Windows серверы, простота |
| **Docker** | Linux | ⭐⭐⭐ | Cloud, контейнеризация |
| **Systemd** | Linux | ⭐⭐ | Linux серверы, стандарт |
| **Nginx + Systemd** | Linux | ⭐⭐⭐ | Production, HTTPS, масштабирование |

**Для syntog.ru** рекомендую: **Systemd + Nginx** (Вариант 5)
- Надёжно
- Production-grade
- HTTPS автоматически
- Healthcheck встроен

---

Какой вариант выбираешь? Помогу настроить! 🚀
