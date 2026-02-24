# MERKLITH Node Monitoring Script
param(
    [int]$RefreshInterval = 5
)

Write-Host "MERKLITH Node Monitor" -ForegroundColor Green
Write-Host "==================" -ForegroundColor Green
Write-Host ""

while ($true) {
    Clear-Host
    $timestamp = Get-Date -Format "HH:mm:ss"
    Write-Host "MERKLITH Node Monitor - $timestamp" -ForegroundColor Cyan
    Write-Host "================================" -ForegroundColor Cyan
    Write-Host ""
    
    # Check processes
    $processes = Get-Process | Where-Object { $_.ProcessName -like "*merklith-node*" }
    Write-Host "Running Nodes: $($processes.Count)" -ForegroundColor Yellow
    
    foreach ($proc in $processes) {
        $port = switch ($proc.Id) {
            176008 { "8545" }
            53204  { "8547" }
            92552  { "8549" }
            default { "Unknown" }
        }
        Write-Host "  Node PID $($proc.Id) (Port: $port) - RAM: $([math]::Round($proc.WorkingSet64/1MB, 2)) MB" -ForegroundColor Green
    }
    Write-Host ""
    
    # Get block numbers
    Write-Host "Block Numbers:" -ForegroundColor Yellow
    try {
        $response1 = Invoke-RestMethod -Uri "http://localhost:8545" -Method POST -ContentType "application/json" -Body '{"jsonrpc":"2.0","method":"merklith_blockNumber","params":[],"id":1}' -TimeoutSec 2
        Write-Host "  Node 1 (8545): Block $($response1.result)" -ForegroundColor Green
    } catch {
        Write-Host "  Node 1 (8545): Offline" -ForegroundColor Red
    }
    
    try {
        $response2 = Invoke-RestMethod -Uri "http://localhost:8547" -Method POST -ContentType "application/json" -Body '{"jsonrpc":"2.0","method":"merklith_blockNumber","params":[],"id":1}' -TimeoutSec 2
        Write-Host "  Node 2 (8547): Block $($response2.result)" -ForegroundColor Green
    } catch {
        Write-Host "  Node 2 (8547): Offline" -ForegroundColor Red
    }
    
    try {
        $response3 = Invoke-RestMethod -Uri "http://localhost:8549" -Method POST -ContentType "application/json" -Body '{"jsonrpc":"2.0","method":"merklith_blockNumber","params":[],"id":1}' -TimeoutSec 2
        Write-Host "  Node 3 (8549): Block $($response3.result)" -ForegroundColor Green
    } catch {
        Write-Host "  Node 3 (8549): Offline" -ForegroundColor Red
    }
    
    Write-Host ""
    Write-Host "Commands:" -ForegroundColor Yellow
    Write-Host "  CTRL+C to exit" -ForegroundColor Gray
    Write-Host "  ./target/release/merklith.exe - CLI tool" -ForegroundColor Gray
    Write-Host "  tail -f logs/node1.log - View logs" -ForegroundColor Gray
    
    Start-Sleep -Seconds $RefreshInterval
}
