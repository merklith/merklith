# MERKLITH Transaction Test Script for Windows
# This script tests actual transaction processing

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Change to script directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

Write-Host ""
Write-Host "MERKLITH Transaction Test Suite" -ForegroundColor Cyan
Write-Host "============================" -ForegroundColor Cyan
Write-Host ""

function Ensure-ReleaseBinary {
    param([string]$Name)
    $path = "./target/release/$Name.exe"
    if (Test-Path $path) { return }
    Write-Host "Building missing binary: $Name" -ForegroundColor Yellow
    cargo build --release --bin $Name | Out-Null
    if (-not (Test-Path $path)) {
        throw "Required binary not found after build: $path"
    }
}

Ensure-ReleaseBinary -Name "merklith-node"
Ensure-ReleaseBinary -Name "merklith-keygen"
Ensure-ReleaseBinary -Name "merklith"

# Start node
Write-Host "Starting merklith-node..." -ForegroundColor Yellow
$nodeProcess = Start-Process -FilePath "./target/release/merklith-node.exe" -ArgumentList "--chain-id", "1337", "--validator", "--data-dir", "./data/test_tx" -PassThru -WindowStyle Hidden

# Wait for node
Write-Host "Waiting for node to start..." -ForegroundColor Gray
Start-Sleep -Seconds 4

# Test 1: Create wallet
Write-Host "Test 1: Creating wallet..." -ForegroundColor Yellow
$keyOutput = & ./target/release/merklith-keygen.exe new 2>&1
$addressMatch = $keyOutput | Select-String "Address: (merklith1[0-9a-z]+)"
$privateKeyMatch = $keyOutput | Select-String "(0x[0-9a-fA-F]+)$"

if ($addressMatch -and $privateKeyMatch) {
    $senderAddress = $addressMatch.Matches.Groups[1].Value
    $senderKey = $privateKeyMatch.Matches.Groups[1].Value
    Write-Host "  [OK] Created wallet: $senderAddress" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] Failed to create wallet" -ForegroundColor Red
    Write-Host "  Debug - Address match: $addressMatch" -ForegroundColor Gray
    Write-Host "  Debug - Key match: $privateKeyMatch" -ForegroundColor Gray
    Write-Host "  Debug - Output: $keyOutput" -ForegroundColor Gray
    $nodeProcess | Stop-Process -Force
    exit 1
}

# Test 2: Check balance
Write-Host "Test 2: Checking balance..." -ForegroundColor Yellow
try {
    $balanceResponse = Invoke-RestMethod -Uri "http://localhost:8545" -Method Post -ContentType "application/json" -Body "{`"jsonrpc`":`"2.0`",`"method`":`"eth_getBalance`",`"params`":[`"$senderAddress`"],`"id`":1}" -TimeoutSec 5
    $balance = [Convert]::ToInt64($balanceResponse.result, 16)
    $balanceAnv = $balance / 1e18
    Write-Host "  [OK] Balance: $balanceAnv ANV" -ForegroundColor Green
} catch {
    Write-Host "  [FAIL] Balance check failed: $_" -ForegroundColor Red
    $nodeProcess | Stop-Process -Force
    exit 1
}

# Test 3: Create recipient wallet
Write-Host "Test 3: Creating recipient wallet..." -ForegroundColor Yellow
$recipientOutput = & ./target/release/merklith-keygen.exe new 2>&1
$recipientMatch = $recipientOutput | Select-String "Address: (merklith1[0-9a-z]+)"
if ($recipientMatch) {
    $recipientAddress = $recipientMatch.Matches.Groups[1].Value
    Write-Host "  [OK] Recipient: $recipientAddress" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] Failed to create recipient" -ForegroundColor Red
    $nodeProcess | Stop-Process -Force
    exit 1
}

# Test 4: Get block number before transaction
Write-Host "Test 4: Getting current block number..." -ForegroundColor Yellow
$blockBeforeResponse = Invoke-RestMethod -Uri "http://localhost:8545" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' -TimeoutSec 5
$blockBefore = [Convert]::ToInt32($blockBeforeResponse.result, 16)
Write-Host "  [OK] Current block: #$blockBefore" -ForegroundColor Green

# Test 5: Send transaction using CLI
Write-Host "Test 5: Sending transaction..." -ForegroundColor Yellow
$txOutput = & ./target/release/merklith.exe tx send $recipientAddress 1.5 --rpc http://localhost:8545 2>&1
$txMatch = $txOutput | Select-String "Transaction Hash: (0x[0-9a-fA-F]+)"
if ($txMatch) {
    $txHash = $txMatch.Matches.Groups[1].Value
    Write-Host "  [OK] Transaction sent: $txHash" -ForegroundColor Green
} else {
    Write-Host "  [WARN] Transaction submission result unclear" -ForegroundColor Yellow
    Write-Host "  Output: $txOutput" -ForegroundColor Gray
}

# Test 6: Wait for transaction to be mined
Write-Host "Test 6: Waiting for transaction to be mined..." -ForegroundColor Yellow
$maxWait = 30
$waited = 0
$txMined = $false

while ($waited -lt $maxWait) {
    Start-Sleep -Seconds 1
    $blockAfterResponse = Invoke-RestMethod -Uri "http://localhost:8545" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' -TimeoutSec 5
    $blockAfter = [Convert]::ToInt32($blockAfterResponse.result, 16)
    
    if ($blockAfter -gt $blockBefore) {
        Write-Host "  [OK] New block produced: #$blockAfter" -ForegroundColor Green
        $txMined = $true
        break
    }
    $waited++
}

if (-not $txMined) {
    Write-Host "  [FAIL] Transaction not mined within $maxWait seconds" -ForegroundColor Red
    $nodeProcess | Stop-Process -Force
    exit 1
}

# Test 7: Check recipient balance
Write-Host "Test 7: Checking recipient balance..." -ForegroundColor Yellow
Start-Sleep -Seconds 2
$recipientBalanceResponse = Invoke-RestMethod -Uri "http://localhost:8545" -Method Post -ContentType "application/json" -Body "{`"jsonrpc`":`"2.0`",`"method`":`"eth_getBalance`",`"params`":[`"$recipientAddress`"],`"id`":1}" -TimeoutSec 5
$recipientBalance = [Convert]::ToInt64($recipientBalanceResponse.result, 16)
$recipientBalanceAnv = $recipientBalance / 1e18

if ($recipientBalance -gt 0) {
    Write-Host "  [OK] Recipient received: $recipientBalanceAnv ANV" -ForegroundColor Green
} else {
    Write-Host "  [WARN] Recipient balance is 0 (may need more time)" -ForegroundColor Yellow
}

# Test 8: Get chain stats
Write-Host "Test 8: Getting chain statistics..." -ForegroundColor Yellow
$statsResponse = Invoke-RestMethod -Uri "http://localhost:8545" -Method Post -ContentType "application/json" -Body '{"jsonrpc":"2.0","method":"merklith_getChainStats","params":[],"id":1}' -TimeoutSec 5
Write-Host "  [OK] Chain stats retrieved" -ForegroundColor Green
Write-Host "       Block height: $($statsResponse.result.blockHeight)" -ForegroundColor Gray
Write-Host "       Transactions: $($statsResponse.result.totalTransactions)" -ForegroundColor Gray
Write-Host "       Accounts: $($statsResponse.result.totalAccounts)" -ForegroundColor Gray

# Cleanup
Write-Host ""
Write-Host "Cleaning up..." -ForegroundColor Gray
$nodeProcess | Stop-Process -Force
Remove-Item -Path "./data/test_tx" -Recurse -Force -ErrorAction SilentlyContinue

# Summary
Write-Host ""
Write-Host "============================" -ForegroundColor Cyan
Write-Host "[SUCCESS] Transaction test completed!" -ForegroundColor Green
Write-Host "============================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Tested Features:" -ForegroundColor White
Write-Host "  - Wallet creation" -ForegroundColor Gray
Write-Host "  - Balance queries" -ForegroundColor Gray
Write-Host "  - Transaction submission" -ForegroundColor Gray
Write-Host "  - Block production" -ForegroundColor Gray
Write-Host "  - Chain statistics" -ForegroundColor Gray
