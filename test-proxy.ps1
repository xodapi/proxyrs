# Тестовый скрипт для opencode-proxy-rs
# Использование: .\test-proxy.ps1

$baseUrl = "http://127.0.0.1:3001"
$testResults = @()

function Test-Endpoint {
    param(
        [string]$Name,
        [string]$Endpoint,
        [string]$Method = "GET",
        [object]$Body = $null
    )
    
    try {
        $url = "$baseUrl$Endpoint"
        $params = @{
            Uri = $url
            Method = $Method
            ErrorAction = "Stop"
            TimeoutSec = 5
        }
        
        if ($Body) {
            $params["ContentType"] = "application/json"
            $params["Body"] = $Body
        }
        
        $response = Invoke-WebRequest @params
        $status = $response.StatusCode
        $success = $true
        $message = "OK ($status)"
    }
    catch {
        $success = $false
        $message = $_.Exception.Message
    }
    
    $result = @{
        Name = $Name
        Endpoint = $Endpoint
        Success = $success
        Message = $message
    }
    
    return $result
}

Write-Host "`n=== OpenCode Proxy Test Suite ===" -ForegroundColor Cyan
Write-Host "Testing: $baseUrl`n" -ForegroundColor Gray

# Test 1: Health Check
$test1 = Test-Endpoint -Name "Health Check" -Endpoint "/health"
$testResults += $test1
Write-Host "✓ Health Check: $($test1.Message)" -ForegroundColor $(if ($test1.Success) { "Green" } else { "Red" })

# Test 2: Models List
$test2 = Test-Endpoint -Name "Models List" -Endpoint "/v1/models"
$testResults += $test2
Write-Host "✓ Models List: $($test2.Message)" -ForegroundColor $(if ($test2.Success) { "Green" } else { "Red" })

# Test 3: Dashboard
$test3 = Test-Endpoint -Name "Dashboard" -Endpoint "/dashboard"
$testResults += $test3
Write-Host "✓ Dashboard: $($test3.Message)" -ForegroundColor $(if ($test3.Success) { "Green" } else { "Red" })

# Test 4: Flow Page
$test4 = Test-Endpoint -Name "Flow" -Endpoint "/flow"
$testResults += $test4
Write-Host "✓ Flow: $($test4.Message)" -ForegroundColor $(if ($test4.Success) { "Green" } else { "Red" })

# Test 5: Playground
$test5 = Test-Endpoint -Name "Playground" -Endpoint "/playground"
$testResults += $test5
Write-Host "✓ Playground: $($test5.Message)" -ForegroundColor $(if ($test5.Success) { "Green" } else { "Red" })

# Test 6: Chat Completions (with 429 retry logic)
$chatBody = @{
    model = "deepseek-v4-flash-free"
    messages = @(@{
        role = "user"
        content = "test"
    })
    max_tokens = 5
} | ConvertTo-Json

$test6 = Test-Endpoint -Name "Chat Completions" -Endpoint "/v1/chat/completions" -Method "POST" -Body $chatBody
$testResults += $test6
Write-Host "✓ Chat Completions: $($test6.Message)" -ForegroundColor $(if ($test6.Success) { "Green" } else { "Red" })

# Test 7: Metrics (protected)
$test7 = Test-Endpoint -Name "Metrics" -Endpoint "/metrics"
$testResults += $test7
Write-Host "✓ Metrics: $($test7.Message)" -ForegroundColor $(if ($test7.Success) { "Green" } else { "Red" })

# Test 8: Diag
$test8 = Test-Endpoint -Name "Diagnostics" -Endpoint "/diag"
$testResults += $test8
Write-Host "✓ Diagnostics: $($test8.Message)" -ForegroundColor $(if ($test8.Success) { "Green" } else { "Red" })

# Summary
$passCount = ($testResults | Where-Object { $_.Success }).Count
$totalCount = $testResults.Count

Write-Host "`n=== Test Summary ===" -ForegroundColor Cyan
Write-Host "Passed: $passCount / $totalCount" -ForegroundColor $(if ($passCount -eq $totalCount) { "Green" } else { "Yellow" })

if ($passCount -eq $totalCount) {
    Write-Host "`n✅ All tests passed! Proxy is working correctly." -ForegroundColor Green
    Write-Host "`nConfiguration for Factory Droid:" -ForegroundColor Cyan
    Write-Host @"
{
  "model": "deepseek-v4-flash-free",
  "id": "custom:opencode-deepseek-v4-flash-free",
  "baseUrl": "http://127.0.0.1:3001/v1",
  "apiKey": "public",
  "provider": "generic-chat-completion-api"
}
"@ -ForegroundColor Gray
} else {
    Write-Host "`n⚠️ Some tests failed. Check proxy is running and accessible." -ForegroundColor Yellow
}

Write-Host ""
