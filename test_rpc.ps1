# MERKLITH Comprehensive RPC Test Suite for Windows
# Tests all RPC endpoints without requiring real transactions

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Change to script directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

Write-Host ""
Write-Host "MERKLITH Comprehensive RPC Test Suite" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan
Write-Host ""

# Start node
Write-Host "Starting merklith-node..." -ForegroundColor Yellow
$dataDir = "./data/rpc_test_$(Get-Random)"
$nodeProcess = Start-Process -FilePath "./target/release/merklith-node.exe" -ArgumentList "--chain-id", "1337", "--validator", "--data-dir", $dataDir -PassThru -WindowStyle Hidden

# Wait for node
Write-Host "Waiting for node to initialize..." -ForegroundColor Gray
Start-Sleep -Seconds 4

$rpcUrl = "http://localhost:8545"
$testsPassed = 0
$testsFailed = 0

function Test-RpcMethod {
    param(
        [string]$Name,
        [string]$Method,
        [array]$Params,
        [scriptblock]$Validator
    )
    
    Write-Host "Testing $Name..." -ForegroundColor Yellow -NoNewline
    
    try {
        $body = @{
            jsonrpc = "2.0"
            method = $Method
            params = $Params
            id = 1
        } | ConvertTo-Json -Compress
        
        $response = Invoke-RestMethod -Uri $rpcUrl -Method Post -ContentType "application/json" -Body $body -TimeoutSec 10
        
        if ($Validator) {
            $result = & $Validator $response.result
            if ($result) {
                Write-Host " [OK]" -ForegroundColor Green
                $script:testsPassed++
            } else {
                Write-Host " [FAIL] Validation failed" -ForegroundColor Red
                Write-Host "    Result: $($response.result)" -ForegroundColor Gray
                $script:testsFailed++
            }
        } else {
            Write-Host " [OK]" -ForegroundColor Green
            $script:testsPassed++
        }
        return $true
    } catch {
        Write-Host " [FAIL] $_" -ForegroundColor Red
        $script:testsFailed++
        return $false
    }
}

# Test 1: Chain ID
Test-RpcMethod -Name "merklith_chainId" -Method "merklith_chainId" -Params @() -Validator {
    param($result)
    $result -eq "0x539"
}

# Test 2: eth_chainId (alias)
Test-RpcMethod -Name "eth_chainId" -Method "eth_chainId" -Params @() -Validator {
    param($result)
    $result -eq "0x539"
}

# Test 3: Block Number
Test-RpcMethod -Name "merklith_blockNumber" -Method "merklith_blockNumber" -Params @() -Validator {
    param($result)
    $blockNum = [Convert]::ToInt32($result, 16)
    $blockNum -gt 0
}

# Test 4: eth_blockNumber (alias)
Test-RpcMethod -Name "eth_blockNumber" -Method "eth_blockNumber" -Params @() -Validator {
    param($result)
    $blockNum = [Convert]::ToInt32($result, 16)
    $blockNum -gt 0
}

# Test 5: Gas Price
Test-RpcMethod -Name "merklith_gasPrice" -Method "merklith_gasPrice" -Params @() -Validator {
    param($result)
    $result -match "^0x[0-9a-f]+$"
}

# Test 6: eth_gasPrice (alias)
Test-RpcMethod -Name "eth_gasPrice" -Method "eth_gasPrice" -Params @() -Validator {
    param($result)
    $result -match "^0x[0-9a-f]+$"
}

# Test 7: Estimate Gas
Test-RpcMethod -Name "merklith_estimateGas" -Method "merklith_estimateGas" -Params @() -Validator {
    param($result)
    $result -match "^0x[0-9a-f]+$"
}

# Test 8: eth_estimateGas (alias)
Test-RpcMethod -Name "eth_estimateGas" -Method "eth_estimateGas" -Params @() -Validator {
    param($result)
    $result -match "^0x[0-9a-f]+$"
}

# Test 9: Syncing Status
Test-RpcMethod -Name "merklith_syncing" -Method "merklith_syncing" -Params @() -Validator {
    param($result)
    $result -eq $false
}

# Test 10: eth_syncing (alias)
Test-RpcMethod -Name "eth_syncing" -Method "eth_syncing" -Params @() -Validator {
    param($result)
    $result -eq $false
}

# Test 11: Protocol Version
Test-RpcMethod -Name "merklith_version" -Method "merklith_version" -Params @() -Validator {
    param($result)
    $result -match "merklith"
}

# Test 12: Get Accounts
Test-RpcMethod -Name "merklith_accounts" -Method "merklith_accounts" -Params @() -Validator {
    param($result)
    $result -is [array]
}

# Test 13: eth_accounts (alias)
Test-RpcMethod -Name "eth_accounts" -Method "eth_accounts" -Params @() -Validator {
    param($result)
    $result -is [array]
}

# Test 14: Get Balance (use zero address)
$zeroAddress = "0x0000000000000000000000000000000000000000"
Test-RpcMethod -Name "merklith_getBalance" -Method "merklith_getBalance" -Params @($zeroAddress) -Validator {
    param($result)
    $result -match "^0x[0-9a-f]+$"
}

# Test 15: eth_getBalance (alias)
Test-RpcMethod -Name "eth_getBalance" -Method "eth_getBalance" -Params @($zeroAddress) -Validator {
    param($result)
    $result -match "^0x[0-9a-f]+$"
}

# Test 16: Get Transaction Count
Test-RpcMethod -Name "merklith_getNonce" -Method "merklith_getNonce" -Params @($zeroAddress) -Validator {
    param($result)
    $result -match "^0x[0-9a-f]+$"
}

# Test 17: eth_getTransactionCount (alias)
Test-RpcMethod -Name "eth_getTransactionCount" -Method "eth_getTransactionCount" -Params @($zeroAddress) -Validator {
    param($result)
    $result -match "^0x[0-9a-f]+$"
}

# Test 18: Get Block By Number (latest)
Test-RpcMethod -Name "merklith_getBlockByNumber (latest)" -Method "merklith_getBlockByNumber" -Params @("latest") -Validator {
    param($result)
    $result.number -match "^0x[0-9a-f]+$" -and $result.hash -match "^0x[0-9a-f]+$"
}

# Test 19: eth_getBlockByNumber (latest)
Test-RpcMethod -Name "eth_getBlockByNumber (latest)" -Method "eth_getBlockByNumber" -Params @("latest") -Validator {
    param($result)
    $result.number -match "^0x[0-9a-f]+$" -and $result.hash -match "^0x[0-9a-f]+$"
}

# Test 20: Get Chain Stats
Test-RpcMethod -Name "merklith_getChainStats" -Method "merklith_getChainStats" -Params @() -Validator {
    param($result)
    # The result can be either a PSCustomObject (has properties) or a hashtable
    $hasBlockNumber = $result.blockNumber -ne $null -or $result.blockHeight -ne $null
    $hasAccounts = $result.accounts -ne $null -or $result.totalAccounts -ne $null
    $hasBlockNumber -and $hasAccounts
}

# Test 21: web3_clientVersion
Test-RpcMethod -Name "web3_clientVersion" -Method "web3_clientVersion" -Params @() -Validator {
    param($result)
    $result -match "merklith"
}

# Test 22: eth_mining
Test-RpcMethod -Name "eth_mining" -Method "eth_mining" -Params @() -Validator {
    param($result)
    $result -eq $true
}

# Test 23: eth_coinbase
Test-RpcMethod -Name "eth_coinbase" -Method "eth_coinbase" -Params @() -Validator {
    param($result)
    $result -match "^0x[0-9a-f]{40}$"
}

# Test 24: net_version
Test-RpcMethod -Name "net_version" -Method "net_version" -Params @() -Validator {
    param($result)
    $result -eq "1337"
}

# Test 25: net_listening
Test-RpcMethod -Name "net_listening" -Method "net_listening" -Params @() -Validator {
    param($result)
    $result -eq $true
}

# Test 26: net_peerCount
Test-RpcMethod -Name "net_peerCount" -Method "net_peerCount" -Params @() -Validator {
    param($result)
    $result -match "^0x[0-9a-f]+$"
}

# Test 27: Get Block Info
Test-RpcMethod -Name "merklith_getBlockInfo" -Method "merklith_getBlockInfo" -Params @("latest") -Validator {
    param($result)
    $result.number -ne $null
}

# Test 28: Get Current Block Hash
Test-RpcMethod -Name "merklith_getCurrentBlockHash" -Method "merklith_getCurrentBlockHash" -Params @() -Validator {
    param($result)
    $result -match "^0x[0-9a-f]+$"
}

# Cleanup
Write-Host ""
Write-Host "Cleaning up..." -ForegroundColor Gray
$nodeProcess | Stop-Process -Force
Remove-Item -Path $dataDir -Recurse -Force -ErrorAction SilentlyContinue

# Summary
Write-Host ""
Write-Host "===================================" -ForegroundColor Cyan
Write-Host "Test Summary:" -ForegroundColor White
Write-Host "  Passed: $testsPassed" -ForegroundColor Green
Write-Host "  Failed: $testsFailed" -ForegroundColor $(if ($testsFailed -gt 0) { "Red" } else { "Green" })
Write-Host "===================================" -ForegroundColor Cyan

if ($testsFailed -eq 0) {
    Write-Host "[SUCCESS] All RPC tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "[FAILURE] Some tests failed!" -ForegroundColor Red
    exit 1
}
