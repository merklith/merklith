# MERKLITH API Reference

Complete reference for the MERKLITH JSON-RPC API.

## Base URL
```
HTTP:  http://localhost:8545
WebSocket: ws://localhost:8546
```

## Authentication

No authentication required for local nodes. For production:
- Use firewall rules to restrict access
- Implement JWT authentication for admin methods
- Use TLS/WSS for encrypted connections

## Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "method_name",
  "params": [param1, param2, ...],
  "id": 1
}
```

## Response Format

### Success
```json
{
  "jsonrpc": "2.0",
  "result": "...",
  "id": 1
}
```

### Error
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Error description"
  },
  "id": 1
}
```

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | JSON is not a valid request object |
| -32601 | Method not found | Method doesn't exist |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Internal JSON-RPC error |
| -32000 | Server error | Generic server error |
| -32001 | Invalid nonce | Transaction nonce mismatch |
| -32002 | Invalid signature | Signature verification failed |
| -32003 | Insufficient balance | Not enough funds |

---

## MERKLITH Methods

### merklith_chainId

Returns the chain ID.

**Parameters**: None

**Returns**: `String` - Chain ID in hex

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_chainId",
    "params": [],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "0x539",
  "id": 1
}
```

---

### merklith_blockNumber

Returns the current block number.

**Parameters**: None

**Returns**: `String` - Block number in hex

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_blockNumber",
    "params": [],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "0x4b1",
  "id": 1
}
```

---

### merklith_getBalance

Returns the balance of an address.

**Parameters**:
1. `address` (string): Address to check
2. `blockNumber` (string, optional): Block number or "latest"

**Returns**: `String` - Balance in wei (hex)

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getBalance",
    "params": ["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0", "latest"],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "0x56bc75e2d63100000",
  "id": 1
}
```

---

### merklith_getNonce

Returns the transaction count (nonce) for an address.

**Parameters**:
1. `address` (string): Address to check
2. `blockNumber` (string, optional): Block number or "latest"

**Returns**: `String` - Nonce in hex

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getNonce",
    "params": ["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"],
    "id": 1
  }'
```

---

### merklith_transfer

Transfers ANV tokens (requires signature).

**Parameters**:
1. `from` (string): Sender address
2. `to` (string): Recipient address
3. `value` (string): Amount in wei (hex)
4. `nonce` (string): Transaction nonce (hex)
5. `signature` (string): Ed25519 signature (hex, 128 chars)
6. `publicKey` (string): Sender's public key (hex, 64 chars)

**Returns**: `String` - Transaction hash

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_transfer",
    "params": [
      "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
      "0x8ba1f109551bD432803012645Hac136c82C3e8c9",
      "0xde0b6b3a7640000",
      "0x0",
      "0x1234567890abcdef...",
      "0xabcdef1234567890..."
    ],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "0xabc123def456...",
  "id": 1
}
```

**Note**: This method requires a valid Ed25519 signature.

---

### merklith_sendSignedTransaction

Submits a pre-signed transaction.

**Parameters**:
1. `signedTx` (string): RLP-encoded signed transaction (hex)

**Returns**: `String` - Transaction hash

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_sendSignedTransaction",
    "params": ["0xf86c8085..."],
    "id": 1
  }'
```

---

### merklith_getBlockByNumber

Returns block information by number.

**Parameters**:
1. `blockNumber` (string): Block number (hex) or "latest", "earliest", "pending"
2. `fullTransactions` (boolean): Include full transaction objects

**Returns**: `Object` - Block data

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getBlockByNumber",
    "params": ["0x4b1", true],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "number": "0x4b1",
    "hash": "0xabc...",
    "parentHash": "0xdef...",
    "timestamp": "0x64b...",
    "transactions": [...]
  },
  "id": 1
}
```

---

### merklith_getTransactionByHash

Returns transaction information by hash.

**Parameters**:
1. `txHash` (string): Transaction hash

**Returns**: `Object` - Transaction data

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getTransactionByHash",
    "params": ["0xabc123..."],
    "id": 1
  }'
```

---

### merklith_gasPrice

Returns current gas price.

**Parameters**: None

**Returns**: `String` - Gas price in wei (hex)

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_gasPrice",
    "params": [],
    "id": 1
  }'
```

---

### merklith_estimateGas

Estimates gas required for transaction.

**Parameters**:
1. `transaction` (object): Transaction object

**Returns**: `String` - Estimated gas (hex)

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_estimateGas",
    "params": [{
      "from": "0x742d...",
      "to": "0x8ba1...",
      "value": "0xde0b6b3a7640000"
    }],
    "id": 1
  }'
```

---

### merklith_accounts

Returns list of accounts in the node.

**Parameters**: None

**Returns**: `Array` - Array of addresses

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_accounts",
    "params": [],
    "id": 1
  }'
```

---

### merklith_createWallet

Creates a new wallet.

**Parameters**: None

**Returns**: `Object` - Address and public key

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_createWallet",
    "params": [],
    "id": 1
  }'
```

---

## Ethereum Compatibility Methods

MERKLITH supports standard Ethereum JSON-RPC methods:

### eth_chainId
Alias for `merklith_chainId`

### eth_blockNumber
Alias for `merklith_blockNumber`

### eth_getBalance
Alias for `merklith_getBalance`

### eth_getTransactionCount
Alias for `merklith_getNonce`

### eth_gasPrice
Alias for `merklith_gasPrice`

### eth_estimateGas
Alias for `merklith_estimateGas`

### eth_getBlockByNumber
Alias for `merklith_getBlockByNumber`

### eth_getTransactionByHash
Alias for `merklith_getTransactionByHash`

### eth_sendRawTransaction
Alias for `merklith_sendSignedTransaction`

### eth_call
Read-only contract call

### eth_getCode
Get contract bytecode

### eth_getStorageAt
Get storage slot value

---

## WebSocket Subscriptions

### Subscribe to New Blocks

```javascript
const ws = new WebSocket('ws://localhost:8546');

ws.onopen = () => {
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'eth_subscribe',
    params: ['newHeads']
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('New block:', data.result);
};
```

### Subscribe to Pending Transactions

```javascript
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  id: 1,
  method: 'eth_subscribe',
  params: ['newPendingTransactions']
}));
```

### Unsubscribe

```javascript
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  id: 1,
  method: 'eth_unsubscribe',
  params: ['subscription_id']
}));
```

---

## Python Example

```python
import requests
import json

class MERKLITHClient:
    def __init__(self, url='http://localhost:8545'):
        self.url = url
    
    def call(self, method, params=None):
        payload = {
            'jsonrpc': '2.0',
            'method': method,
            'params': params or [],
            'id': 1
        }
        response = requests.post(self.url, json=payload)
        return response.json()['result']
    
    def get_balance(self, address):
        return self.call('merklith_getBalance', [address, 'latest'])
    
    def get_block_number(self):
        return self.call('merklith_blockNumber')

# Usage
client = MERKLITHClient()
balance = client.get_balance('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0')
print(f"Balance: {balance}")
```

---

## JavaScript/Node.js Example

```javascript
const axios = require('axios');

class MERKLITHClient {
  constructor(url = 'http://localhost:8545') {
    this.url = url;
  }

  async call(method, params = []) {
    const response = await axios.post(this.url, {
      jsonrpc: '2.0',
      method,
      params,
      id: 1
    });
    return response.data.result;
  }

  async getBalance(address) {
    return this.call('merklith_getBalance', [address, 'latest']);
  }

  async getBlockNumber() {
    return this.call('merklith_blockNumber');
  }
}

// Usage
const client = new MERKLITHClient();
client.getBalance('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0')
  .then(balance => console.log('Balance:', balance));
```

---

## Rate Limits

- **Default**: 100 requests per minute per IP
- **Burst**: 10 requests per second
- **WebSocket**: 1000 messages per minute

To increase limits, modify node configuration:
```toml
[rpc]
rate_limit = 1000  # requests per minute
rate_limit_burst = 100  # burst capacity
```

---

## See Also

- [Intermediate Technical Documentation](../intermediate/01-technical-overview.md)
- [Whitepaper](../whitepaper/MERKLITH_WHITEPAPER.md)
- [CLI Guide](../CLI_GUIDE.md)