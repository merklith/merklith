# MERKLITH Web Ecosystem

Complete web-based blockchain ecosystem for MERKLITH including block explorer, web wallet, and API services.

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     User Interface Layer                     │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │ Block Explorer   │  │ Web Wallet       │                 │
│  │ (Port 3000)      │  │ (Port 3001)      │                 │
│  └────────┬─────────┘  └────────┬─────────┘                 │
└───────────┼────────────────────┼────────────────────────────┘
            │                    │
            └────────┬───────────┘
                     │
┌────────────────────┼───────────────────────────────────────┐
│                    │    API Layer                          │
│           ┌────────▼─────────┐                             │
│           │   Web API        │                             │
│           │   (Port 3002)    │                             │
│           └────────┬─────────┘                             │
│                    │                                       │
│           ┌────────▼─────────┐                             │
│           │   WebSocket      │                             │
│           │   Real-time      │                             │
│           └──────────────────┘                             │
└─────────────────────────────────────────────────────────────┘
                     │
            ┌────────▼─────────┐
            │  MERKLITH Node      │
            │  (RPC: 8545)     │
            └──────────────────┘
```

## Components

### 1. Block Explorer (merklith-explorer)

Modern React-based blockchain explorer.

**Features:**
- Real-time block updates
- Transaction details and traces
- Account balance and history
- Validator set and statistics
- Network analytics and charts
- Advanced search functionality
- Responsive design

**Pages:**
- `/` - Dashboard with network overview
- `/blocks` - Block list
- `/block/:number` - Block details
- `/transactions` - Transaction list
- `/tx/:hash` - Transaction details
- `/address/:address` - Account details
- `/validators` - Validator list
- `/stats` - Network statistics
- `/api` - API documentation

### 2. Web Wallet (merklith-wallet)

Secure browser-based wallet application.

**Features:**
- Create new wallet
- Import from mnemonic/private key
- Send/Receive ANV tokens
- Transaction history
- Multiple account support
- Hardware wallet integration (Ledger/Trezor)
- MetaMask compatibility
- QR code support

**Pages:**
- `/` - Dashboard
- `/create` - Create new wallet
- `/import` - Import existing wallet
- `/send` - Send transaction
- `/receive` - Receive tokens
- `/history` - Transaction history
- `/settings` - Wallet settings

### 3. Web API (merklith-web-api)

High-performance Rust API backend.

**Features:**
- RESTful API endpoints
- WebSocket real-time updates
- PostgreSQL database integration
- Redis caching layer
- Rate limiting
- CORS support
- Request logging

**Endpoints:**

#### Blocks
- `GET /api/blocks` - List blocks
- `GET /api/blocks/latest` - Get latest block
- `GET /api/blocks/:number` - Get block by number
- `GET /api/blocks/hash/:hash` - Get block by hash

#### Transactions
- `GET /api/transactions` - List transactions
- `GET /api/transactions/:hash` - Get transaction
- `GET /api/transactions/pending` - Pending transactions

#### Accounts
- `GET /api/accounts/:address` - Get account
- `GET /api/accounts/:address/transactions` - Account transactions
- `GET /api/accounts/:address/balance` - Account balance

#### Validators
- `GET /api/validators` - List validators
- `GET /api/validators/:address` - Get validator
- `GET /api/validators/stats` - Validator statistics

#### Network
- `GET /api/network/stats` - Network statistics
- `GET /api/network/peers` - Connected peers
- `GET /api/network/syncing` - Sync status

#### WebSocket
- `WS /ws` - Real-time updates

## Installation

### Prerequisites
- Docker 20.10+
- Docker Compose 2.0+
- 4GB RAM minimum
- 10GB free disk space

### Quick Start

```bash
# Clone repository
git clone https://github.com/merklithnetwork/merklith.git
cd merklith/web

# Start all services
docker-compose up -d

# Wait for services to be ready
./scripts/wait-for-services.sh

# Access services
open http://localhost:3000      # Block Explorer
open http://localhost:3001      # Web Wallet
open http://localhost:3002      # API
```

### Production Deployment

```bash
# Copy environment variables
cp .env.example .env

# Edit configuration
nano .env

# Start in production mode
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# SSL certificates (Let's Encrypt)
./scripts/setup-ssl.sh
```

## Development

### Local Development

```bash
# Start backend services
docker-compose up -d postgres redis merklith-node

# Install API dependencies
cd api
cargo build

# Run API server
cargo run

# In another terminal - Explorer
cd explorer
npm install
npm start

# In another terminal - Wallet
cd wallet
npm install
npm start
```

### API Testing

```bash
# Get latest block
curl http://localhost:3002/api/blocks/latest

# Get account balance
curl http://localhost:3002/api/accounts/merklith1qxy.../balance

# WebSocket connection
wscat -c ws://localhost:3002/ws
```

## Configuration

### Environment Variables

```env
# Node Configuration
MERKLITH_RPC_URL=http://merklith-node:8545
MERKLITH_CHAIN_ID=1337

# Database
DATABASE_URL=postgres://merklith:merklith@postgres:5432/merklith

# Redis
REDIS_URL=redis://redis:6379

# API
API_PORT=3002
API_RATE_LIMIT=1000

# Frontend
REACT_APP_API_URL=http://localhost:3002
REACT_APP_RPC_URL=http://localhost:8545
```

### Custom Domain Setup

```nginx
# Add to /etc/hosts for local development
127.0.0.1 localhost
```

## Architecture

### Data Flow

```
1. Block Production
   MERKLITH Node → WebSocket → API Cache → Clients

2. User Transaction
   Web Wallet → API → MERKLITH Node → Block → Explorer Update

3. Account Query
   Client → API → Cache/DB → Response

4. Real-time Updates
   MERKLITH Node → WebSocket → Redis Pub/Sub → Clients
```

### Database Schema

```sql
-- Blocks table
CREATE TABLE blocks (
    number BIGINT PRIMARY KEY,
    hash VARCHAR(66) UNIQUE,
    parent_hash VARCHAR(66),
    timestamp TIMESTAMP,
    proposer VARCHAR(42),
    tx_count INTEGER,
    gas_used BIGINT,
    gas_limit BIGINT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Transactions table
CREATE TABLE transactions (
    hash VARCHAR(66) PRIMARY KEY,
    block_number BIGINT REFERENCES blocks(number),
    from_address VARCHAR(42),
    to_address VARCHAR(42),
    value NUMERIC,
    gas_price BIGINT,
    gas_used BIGINT,
    nonce BIGINT,
    status INTEGER,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Accounts table
CREATE TABLE accounts (
    address VARCHAR(42) PRIMARY KEY,
    balance NUMERIC DEFAULT 0,
    nonce BIGINT DEFAULT 0,
    code TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

## API Examples

### JavaScript/TypeScript

```typescript
// Get latest blocks
const blocks = await fetch('http://localhost:3002/api/blocks')
  .then(r => r.json());

// Get account balance
const balance = await fetch('http://localhost:3002/api/accounts/merklith1.../balance')
  .then(r => r.json());

// WebSocket connection
const ws = new WebSocket('ws://localhost:3002/ws');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('New block:', data.block);
};
```

### Python

```python
import requests
import websocket

# Get block
block = requests.get('http://localhost:3002/api/blocks/12345').json()

# WebSocket
def on_message(ws, message):
    print(f"Received: {message}")

ws = websocket.WebSocketApp("ws://localhost:3002/ws",
                            on_message=on_message)
ws.run_forever()
```

### cURL

```bash
# Get network stats
curl http://localhost:3002/api/network/stats

# Search
curl "http://localhost:3002/api/search?q=0xabc..."

# Get validator set
curl http://localhost:3002/api/validators
```

## Monitoring

### Health Checks

```bash
# Node health
curl http://localhost:8545 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_syncing","params":[],"id":1}'

# API health
curl http://localhost:3002/health

# Full stack
docker-compose ps
```

### Logs

```bash
# View all logs
docker-compose logs -f

# Specific service
docker-compose logs -f merklith-node
docker-compose logs -f merklith-api
docker-compose logs -f merklith-explorer
```

## Security

### Wallet Security

- **Client-side encryption**: Private keys never leave browser
- **Secure storage**: Encrypted localStorage with password
- **Hardware wallets**: Ledger/Trezor support
- **Transaction signing**: User confirmation required
- **Session timeout**: Auto-lock after inactivity

### API Security

- **Rate limiting**: 1000 requests/minute per IP
- **CORS**: Configured for specific origins
- **Input validation**: All inputs sanitized
- **SQL injection protection**: Parameterized queries
- **No sensitive data**: Private keys never sent to server

### Infrastructure Security

- **Container isolation**: Each service in separate container
- **Network segmentation**: Internal network for services
- **SSL/TLS**: HTTPS in production
- **Secrets management**: Environment variables for secrets

## Troubleshooting

### Common Issues

**Services not starting:**
```bash
# Check ports
lsof -i :8545 :3000 :3001 :3002

# Restart services
docker-compose down
docker-compose up -d

# Check logs
docker-compose logs merklith-node
```

**Explorer not loading:**
```bash
# Check API connectivity
curl http://localhost:3002/health

# Rebuild explorer
docker-compose build merklith-explorer
```

**Wallet connection issues:**
```bash
# Check MetaMask network
# Ensure RPC URL: http://localhost:8545
# Chain ID: 1337
```

### Performance Tuning

```bash
# Increase cache size
REDIS_MAXMEMORY=512mb

# Database optimization
POSTGRES_SHARED_BUFFERS=256MB

# API workers
API_WORKERS=4
```

## Development Roadmap

### Phase 1: Core (Complete ✓)
- [x] Block explorer
- [x] Web wallet
- [x] REST API
- [x] WebSocket support
- [x] Docker deployment

### Phase 2: Features
- [ ] Mobile responsive wallet
- [ ] Multi-sig support
- [ ] Staking interface
- [ ] Governance voting
- [ ] Contract verification

### Phase 3: Advanced
- [ ] Cross-chain bridge UI
- [ ] DeFi dashboard
- [ ] Analytics platform
- [ ] Mobile apps
- [ ] Hardware wallet support

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines.

## License

Apache 2.0 OR MIT - See [LICENSE](../LICENSE)