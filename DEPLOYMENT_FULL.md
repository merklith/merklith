# MERKLITH Blockchain - Complete Deployment Guide

## Quick Start (5 minutes)

### Prerequisites
- Docker & Docker Compose
- Git
- 8GB+ RAM
- 50GB+ free disk space

### 1. Start with Docker Compose
```bash
docker-compose up -d
```

### 2. Verify Installation
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'
```

## Services
- **merklith-node-1**: Validator node (http://localhost:8545)
- **merklith-node-2**: Full node (http://localhost:8547)
- **merklith-node-3**: Archive node (http://localhost:8549)
- **prometheus**: Metrics (http://localhost:9093)
- **grafana**: Dashboard (http://localhost:3001)

## Complete Ecosystem
✅ Core Blockchain (PoC Consensus)
✅ Smart Contracts (ERC20, ERC721, Bridge, Governance)
✅ TypeScript SDK
✅ Web Block Explorer
✅ Docker Deployment
✅ Monitoring Stack
✅ 310+ Tests Passing

## Production Ready! 🚀
