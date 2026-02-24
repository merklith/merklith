#!/bin/bash
# MERKLITH Blockchain Auto-Activity Script
# Bu script sürekli transaction üreterek block hareketi yaratır

set -e

MERKLITH_CLI="./target/release/merklith.exe"
RPC_URL="http://localhost:8545"
FAUCET_ACCOUNT="0xFaucet0000000000000000000000000000000000"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 MERKLITH Auto-Activity Bot${NC}"
echo "=================================="
echo ""

# Check if CLI exists
if [ ! -f "$MERKLITH_CLI" ]; then
    echo -e "${RED}❌ merklith.exe bulunamadı!${NC}"
    echo "Önce build edin: cargo build --release -p merklith-cli"
    exit 1
fi

# Target addresses
ADDRESSES=(
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
    "0x8ba1f109551bD432803012645Ac136ddd64DBA72"
    "0xdD870fA1b7C4700F2BD7f44238821C26f7392148"
    "0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B"
    "0x1aB489E589De6E2F9c9b6B9e2F2b1a4c3d5E6F78"
)

TX_COUNT=0
START_TIME=$(date +%s)

echo -e "${BLUE}💳 Faucet Account:${NC} $FAUCET_ACCOUNT"
echo -e "${BLUE}🎯 Target Addresses:${NC} ${#ADDRESSES[@]}"
echo -e "${BLUE}⏱️  Interval:${NC} Random 5-25 seconds"
echo -e "${BLUE}💸 Amount:${NC} Random 0.001-0.1 ANV"
echo -e "${BLUE}🔗 RPC:${NC} $RPC_URL"
echo ""
echo -e "${YELLOW}🔄 Starting transfers... (Ctrl+C to stop)${NC}"
echo ""

# Function to generate random number
random() {
    local min=$1
    local max=$2
    echo $(( $min + RANDOM % ($max - $min + 1) ))
}

# Function to send transaction
send_tx() {
    local to=$1
    local amount=$2
    
    # Use curl to send transaction via RPC
    local response=$(curl -s -X POST "$RPC_URL" \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"method\": \"merklith_transfer\",
            \"params\": [\"$FAUCET_ACCOUNT\", \"$to\", \"$amount\"],
            \"id\": $TX_COUNT
        }" 2>/dev/null)
    
    echo "$response"
}

# Main loop
trap 'echo -e "\n\n${YELLOW}🛑 Bot stopped by user${NC}"; exit 0' INT

while true; do
    # Select random target
    INDEX=$(( RANDOM % ${#ADDRESSES[@]} ))
    TARGET="${ADDRESSES[$INDEX]}"
    
    # Generate random amount (0.001 - 0.1 ANV in wei)
    # 0.001 ANV = 1000000000000000 wei
    # 0.1 ANV = 100000000000000000 wei
    AMOUNT_WEI=$(random 1000000000000000 100000000000000000)
    AMOUNT_ANV=$(echo "scale=6; $AMOUNT_WEI / 1000000000000000000" | bc)
    
    CURRENT_TIME=$(date '+%H:%M:%S')
    echo -n "[$CURRENT_TIME] 💸 Sending ${AMOUNT_ANV} ANV → ${TARGET:0:20}... "
    
    # Send transaction
    RESULT=$(send_tx "$TARGET" "0x$(printf '%x' $AMOUNT_WEI)")
    
    if echo "$RESULT" | grep -q '"result"'; then
        TX_HASH=$(echo "$RESULT" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
        echo -e "${GREEN}✅${NC} ${TX_HASH:0:20}..."
        ((TX_COUNT++))
    else
        ERROR=$(echo "$RESULT" | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
        echo -e "${RED}❌${NC} ${ERROR:-Unknown error}"
    fi
    
    # Stats every 10 transactions
    if [ $(( TX_COUNT % 10 )) -eq 0 ] && [ $TX_COUNT -ne 0 ]; then
        CURRENT_TIME=$(date +%s)
        ELAPSED=$(( CURRENT_TIME - START_TIME ))
        TX_PER_HOUR=$(echo "scale=2; $TX_COUNT * 3600 / $ELAPSED" | bc)
        
        echo ""
        echo -e "${BLUE}📊 Stats:${NC} $TX_COUNT transactions | $TX_PER_HOUR tx/hour | Running: ${ELAPSED}s"
        echo ""
    fi
    
    # Random wait time (5-25 seconds)
    WAIT_TIME=$(random 5 25)
    sleep $WAIT_TIME
done
