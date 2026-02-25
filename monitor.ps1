# MERKLITH Node Monitoring Script
param(
    [int]$RefreshInterval = 5,
    [int[]]$RpcPorts = @(8545, 8547, 8549)
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
        $listeningPorts = @(Get-NetTCPConnection -State Listen -OwningProcess $proc.Id -ErrorAction SilentlyContinue |
            Select-Object -ExpandProperty LocalPort -Unique |
            Sort-Object)
        $portLabel = if ($listeningPorts.Count -gt 0) { ($listeningPorts -join ",") } else { "Unknown" }
        Write-Host "  Node PID $($proc.Id) (Port: $portLabel) - RAM: $([math]::Round($proc.WorkingSet64/1MB, 2)) MB" -ForegroundColor Green
    }
    Write-Host ""
    
    # Get block numbers
    Write-Host "Block Numbers:" -ForegroundColor Yellow
    for ($i = 0; $i -lt $RpcPorts.Count; $i++) {
        $port = $RpcPorts[$i]
        try {
            $response = Invoke-RestMethod -Uri "http://localhost:$port" -Method POST -ContentType "application/json" -Body '{"jsonrpc":"2.0","method":"merklith_blockNumber","params":[],"id":1}' -TimeoutSec 2
            Write-Host "  Node $($i + 1) ($port): Block $($response.result)" -ForegroundColor Green
        } catch {
            Write-Host "  Node $($i + 1) ($port): Offline" -ForegroundColor Red
        }
    }
    
    Write-Host ""
    Write-Host "Commands:" -ForegroundColor Yellow
    Write-Host "  CTRL+C to exit" -ForegroundColor Gray
    Write-Host "  ./target/release/merklith.exe - CLI tool" -ForegroundColor Gray
    Write-Host "  tail -f logs/node1.log - View logs" -ForegroundColor Gray
    
    Start-Sleep -Seconds $RefreshInterval
}
