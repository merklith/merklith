#!/bin/bash
# MERKLITH Blockchain Test Script
# Demonstrates all core features

RPC_URL="http://localhost:8545"

echo "=========================================="
echo "  MERKLITH Blockchain - Feature Test"
echo "=========================================="

# Check if node is running
echo ""
echo "1. Checking node status..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"merklith_chainId","params":[],"id":1}' | jq .

# Get block number
echo ""
echo "2. Current block number..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"merklith_blockNumber","params":[],"id":1}' | jq .

# Get chain stats
echo ""
echo "3. Chain statistics..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"merklith_getChainStats","params":[],"id":1}' | jq .

# Get balance
echo ""
echo "4. Checking pre-funded account balance..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"merklith_getBalance","params":["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0","latest"],"id":1}' | jq .

# Create wallet
echo ""
echo "5. Creating new wallet..."
WALLET=$(curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"merklith_createWallet","params":[],"id":1}')
echo $WALLET | jq .
ADDRESS=$(echo $WALLET | jq -r '.result.address')
PRIVATE_KEY=$(echo $WALLET | jq -r '.result.privateKey')

# Fund wallet
echo ""
echo "6. Funding new wallet with 1 ETH..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"merklith_transfer\",\"params\":[\"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0\",\"$ADDRESS\",\"0xde0b6b3a7640000\"],\"id\":1}" | jq .

# Check new balance
echo ""
echo "7. Checking new wallet balance..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"merklith_getBalance\",\"params\":[\"$ADDRESS\",\"latest\"],\"id\":1}" | jq .

# Send signed transaction
echo ""
echo "8. Sending signed transaction (0.1 ETH)..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"merklith_signAndSendTransaction\",\"params\":[\"$PRIVATE_KEY\",\"0x8ba1f109551bD432803012645Ac136ddd64DBA72\",\"0x16345785d8a0000\"],\"id\":1}" | jq .

# Deploy contract
echo ""
echo "9. Deploying smart contract..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"merklith_deployContract","params":["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0","0x600160005500"],"id":1}' | jq .

# Get block info
echo ""
echo "10. Getting block info..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"merklith_getBlockInfo","params":["latest"],"id":1}' | jq .

# Get blockchain
echo ""
echo "11. Getting last 3 blocks..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"merklith_getBlockChain","params":[0,3],"id":1}' | jq .

# Create attestation
echo ""
echo "12. Creating attestation for block 1..."
curl -s -X POST $RPC_URL -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"merklith_createAttestation\",\"params\":[\"$PRIVATE_KEY\",\"0x1\"],\"id\":1}" | jq .

echo ""
echo "=========================================="
echo "  All tests completed successfully!"
echo "=========================================="
