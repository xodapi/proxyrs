# Все-в-одном: сборка, запуск и тестирование proxy
# Использование: .\start-and-test.ps1

Write-Host "🔨 OpenCode Proxy - Build & Test" -ForegroundColor Cyan
Write-Host ""

# 1. Сборка
Write-Host "Step 1: Building release binary..." -ForegroundColor Yellow
cargo build --release 2>&1 | Select-Object -Last 5
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Build successful" -ForegroundColor Green
Write-Host ""

# 2. Остановить старый процесс
Write-Host "Step 2: Stopping old processes..." -ForegroundColor Yellow
Get-Process opencode-proxy -ErrorAction SilentlyContinue | Stop-Process -Force 2>&1
Start-Sleep -Seconds 2
Write-Host "✅ Old processes stopped" -ForegroundColor Green
Write-Host ""

# 3. Запустить proxy
Write-Host "Step 3: Starting proxy..." -ForegroundColor Yellow
.\target\release\opencode-proxy.exe 2>&1 &
Start-Sleep -Seconds 4
Write-Host "✅ Proxy started" -ForegroundColor Green
Write-Host ""

# 4. Тестировать
Write-Host "Step 4: Running tests..." -ForegroundColor Yellow
Write-Host ""
.\test-proxy.ps1
