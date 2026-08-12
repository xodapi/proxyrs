# Запуск и мониторинг proxy с автоперезапуском
# Использование: .\run-proxy.ps1

$proxyPath = "$PSScriptRoot\target\release\opencode-proxy.exe"
$maxRestarts = 10
$restartCount = 0

if (-not (Test-Path $proxyPath)) {
    Write-Host "❌ Proxy binary not found at: $proxyPath" -ForegroundColor Red
    Write-Host "Run: cargo build --release" -ForegroundColor Yellow
    exit 1
}

Write-Host "🚀 OpenCode Proxy Monitor" -ForegroundColor Cyan
Write-Host "Binary: $proxyPath" -ForegroundColor Gray
Write-Host ""

while ($restartCount -lt $maxRestarts) {
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Starting proxy..." -ForegroundColor Yellow
    
    $process = Start-Process -FilePath $proxyPath `
        -NoNewWindow `
        -PassThru `
        -RedirectStandardOutput "$PSScriptRoot\proxy.log" `
        -RedirectStandardError "$PSScriptRoot\proxy.err.log"
    
    $pid = $process.Id
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] ✅ Proxy started (PID: $pid)" -ForegroundColor Green
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Listening on http://127.0.0.1:3001" -ForegroundColor Green
    Write-Host ""
    
    # Проверяем health каждые 5 секунд
    $healthCheckInterval = 5
    $healthCheckCount = 0
    $maxHealthChecks = 300  # 25 минут
    
    while ($healthCheckCount -lt $maxHealthChecks) {
        Start-Sleep -Seconds $healthCheckInterval
        $healthCheckCount++
        
        try {
            $health = Invoke-WebRequest -Uri "http://127.0.0.1:3001/health" `
                -TimeoutSec 3 `
                -ErrorAction Stop
            
            if ($health.StatusCode -eq 200) {
                # Proxy жив, продолжаем
                if ($healthCheckCount % 12 -eq 0) {  # Выводим каждую минуту
                    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] ✅ Proxy healthy" -ForegroundColor Green
                }
            }
        }
        catch {
            # Proxy не отвечает
            if ($process.HasExited) {
                Write-Host "[$(Get-Date -Format 'HH:mm:ss')] ⚠️ Proxy crashed!" -ForegroundColor Red
                $restartCount++
                if ($restartCount -lt $maxRestarts) {
                    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Restarting... (attempt $restartCount/$maxRestarts)" -ForegroundColor Yellow
                    Start-Sleep -Seconds 2
                }
                break
            }
        }
    }
    
    if ($process -and -not $process.HasExited) {
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Stopping proxy (timeout)..." -ForegroundColor Yellow
        $process | Stop-Process -Force -ErrorAction SilentlyContinue
    }
}

Write-Host ""
Write-Host "❌ Max restarts reached ($maxRestarts)" -ForegroundColor Red
Write-Host "Logs: $PSScriptRoot\proxy.log" -ForegroundColor Gray
