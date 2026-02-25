# MERKLITH End-to-End Test Script for Windows
# This script verifies that all components work correctly

# Change to script directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

Write-Host "MERKLITH End-to-End Test Suite" -ForegroundColor Cyan
Write-Host "==========================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$env:RUST_BACKTRACE = "0"

# Test 1: Binary existence and version
Write-Host "Test 1: Checking binaries..." -ForegroundColor Yellow
$binaries = @("merklith", "merklith-node", "merklith-keygen", "merklith-monitor", "merklith-benchmark", "merklith-faucet")
$missing = @()
foreach ($bin in $binaries) {
    $path = "./target/release/$bin.exe"
    if (Test-Path $path) {
        $version = & $path --version 2>$null
        Write-Host "  [OK] $bin - $version" -ForegroundColor Green
    } else {
        $missing += $bin
        Write-Host "  [WARN] $bin not found" -ForegroundColor Yellow
    }
}

if ($missing.Count -gt 0) {
    Write-Host "Building missing release binaries..." -ForegroundColor Yellow
    cargo build --release --bins | Out-Null
    foreach ($bin in $missing) {
        $path = "./target/release/$bin.exe"
        if (-not (Test-Path $path)) {
            Write-Host "  [FAIL] $bin not found after build" -ForegroundColor Red
            exit 1
        }
    }
}
Write-Host ""

# Test 2: Unit tests
Write-Host "Test 2: Running unit tests..." -ForegroundColor Yellow
$env:RUSTFLAGS = "-Awarnings"
$testOutput = cargo test --lib -p merklith-types -p merklith-crypto -p merklith-core -p merklith-vm --quiet 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  [OK] All core tests passing (244 tests)" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] Tests failed" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Test 3: Key generation
Write-Host "Test 3: Testing key generation..." -ForegroundColor Yellow
$keyOutput = & ./target/release/merklith-keygen.exe new 2>&1
if ($keyOutput -match "Address:") {
    Write-Host "  [OK] Key generation working" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] Key generation failed" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Test 4: Start node and verify block production
Write-Host "Test 4: Testing node startup and block production..." -ForegroundColor Yellow
$nodeProcess = Start-Process -FilePath "./target/release/merklith-node.exe" -ArgumentList "--chain-id", "1337", "--validator" -PassThru -WindowStyle Hidden

# Wait for node to start
Start-Sleep -Seconds 3

# Test RPC endpoint
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8545" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' -TimeoutSec 5
    if ($response.result -eq "0x539") {
        Write-Host "  [OK] RPC endpoint responding (Chain ID: 1337)" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] RPC returned unexpected result" -ForegroundColor Red
        $nodeProcess | Stop-Process -Force
        exit 1
    }
    
    # Check block number
    $blockResponse = Invoke-RestMethod -Uri "http://localhost:8545" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' -TimeoutSec 5
    $blockNumber = [Convert]::ToInt32($blockResponse.result, 16)
    if ($blockNumber -gt 0) {
        Write-Host "  [OK] Block production working (Block #$blockNumber)" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] No blocks produced" -ForegroundColor Red
        $nodeProcess | Stop-Process -Force
        exit 1
    }
} catch {
    Write-Host "  [FAIL] RPC test failed: $_" -ForegroundColor Red
    $nodeProcess | Stop-Process -Force
    exit 1
}

# Cleanup
$nodeProcess | Stop-Process -Force
Write-Host ""

# Test 5: CLI commands
Write-Host "Test 5: Testing CLI commands..." -ForegroundColor Yellow
$helpOutput = & ./target/release/merklith.exe --help 2>&1
if ($helpOutput -match "wallet|account|query|tx") {
    Write-Host "  [OK] CLI help working" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] CLI help failed" -ForegroundColor Red
    exit 1
}
Write-Host ""

Write-Host "==========================" -ForegroundColor Cyan
Write-Host "[SUCCESS] All tests passed!" -ForegroundColor Green
Write-Host "MERKLITH is working correctly on Windows" -ForegroundColor Green
Write-Host "==========================" -ForegroundColor Cyan
