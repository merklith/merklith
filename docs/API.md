# MERKLITH JSON-RPC API

Complete reference for the MERKLITH JSON-RPC API.

## Overview

MERKLITH provides a JSON-RPC 2.0 compatible API for interacting with the blockchain. The API uses MERKLITH-native methods (`merklith_*`) as the primary interface, with a small set of Ethereum-compatible aliases (`eth_*`, `web3_*`, `net_*`) for tool compatibility (e.g., MetaMask).

## Base URL

```
http://localhost:8545
```

## Request Format

All requests follow JSON-RPC 2.0 specification:

```json
{
  "jsonrpc": "2.0",
  "method": "merklith_chainId",
  "params": [],
  "id": 1
}
```

## Response Format

### Success Response

```json
{
  "jsonrpc": "2.0",
  "result": "0x1",
  "id": 1
}
```

### Error Response

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32601,
    "message": "Method not found"
  },
  "id": 1
}
```

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid request | JSON is not a valid request object |
| -32601 | Method not found | Method does not exist |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Internal JSON-RPC error |
| -32000 | Server error | Generic server error |
| -32001 | Not found | Block or resource not found |
| -32002 | Invalid signature | Signature verification failed |
| -32003 | BLS key error | BLS key generation/signing error |

## Quick Reference

### Chain Methods

- `merklith_chainId` - Get chain ID
- `merklith_blockNumber` - Get latest block number
- `merklith_getBlockByNumber` - Get block by number
- `merklith_getBlockInfo` - Get extended block information
- `merklith_getCurrentBlockHash` - Get current block hash
- `merklith_getBlockChain` - Get a range of blocks
- `merklith_getChainStats` - Get chain statistics
- `merklith_syncing` - Check sync status
- `merklith_version` - Get node version
- `merklith_gasPrice` - Get gas price
- `merklith_estimateGas` - Estimate gas

### Account Methods

- `merklith_getBalance` - Get account balance
- `merklith_getNonce` - Get transaction count (nonce)
- `merklith_accounts` - List all accounts
- `merklith_createWallet` - Create a new wallet

### Transaction Methods

- `merklith_transfer` - Transfer funds (unsigned, dev only)
- `merklith_sendSignedTransaction` - Send signed transaction with signature verification
- `merklith_signAndSendTransaction` - Sign and send in one step
- `merklith_sendRawTransaction` - Send raw transaction
- `merklith_getTransactionByHash` - Get transaction by hash

### Contract Methods

- `merklith_deployContract` - Deploy a contract
- `merklith_getCode` - Get contract bytecode
- `merklith_getStorageAt` - Get storage value
- `merklith_call` - Call contract (read-only)

### Consensus Methods

- `merklith_createAttestation` - Submit a BLS attestation

### Ethereum Compatibility Aliases

- `eth_chainId` - Alias for `merklith_chainId`
- `eth_blockNumber` - Alias for `merklith_blockNumber`
- `eth_getBalance` - Alias for `merklith_getBalance`
- `net_version` - Returns chain ID as string
- `web3_clientVersion` - Returns `merklith/0.1.0`

## Detailed Method Reference

---

### merklith_chainId

Returns the chain ID of the current network.

**Parameters**: None

**Returns**: `STRING` - Chain ID as hex string

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
  "result": "0x4269",
  "id": 1
}
```

---

### merklith_blockNumber

Returns the number of the most recent block.

**Parameters**: None

**Returns**: `STRING` - Block number as hex string

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
  "result": "0x3039",
  "id": 1
}
```

---

### merklith_getBalance

Returns the balance of an address.

**Parameters**:
1. `STRING` - Address (0x-prefixed hex)

**Returns**: `STRING` - Balance in wei (hex)

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getBalance",
    "params": ["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "56bc75e2d63100000",
  "id": 1
}
```

**Note**: Result is in wei (10^18 wei = 1 ANV)

---

### merklith_getNonce

Returns the transaction count (nonce) for an address.

**Parameters**:
1. `STRING` - Address (0x-prefixed hex)

**Returns**: `STRING` - Nonce as hex string

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getNonce",
    "params": ["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "0x2a",
  "id": 1
}
```

---

### merklith_transfer

Transfers funds between accounts (unsigned, development use).

**Parameters**:
1. `STRING` - From address
2. `STRING` - To address
3. `STRING` - Amount in wei (hex or decimal)

**Returns**: `STRING` - Transaction hash

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_transfer",
    "params": [
      "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
      "0x8ba1f109551bD432803012645Hac136c82C3e82",
      "0xde0b6b3a7640000"
    ],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "0xabc123def456789...",
  "id": 1
}
```

---

### merklith_sendSignedTransaction

Submits a signed transaction with Ed25519 signature verification.

**Parameters**:
1. `STRING` - From address
2. `STRING` - To address
3. `STRING` - Amount in wei (hex or decimal)
4. `STRING` - Nonce (hex or decimal)
5. `STRING` - Ed25519 signature (0x-prefixed, 64 bytes)
6. `STRING` - Ed25519 public key (0x-prefixed, 32 bytes)

**Returns**: `STRING` - Transaction hash

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_sendSignedTransaction",
    "params": [
      "0xFromAddress...",
      "0xToAddress...",
      "0xde0b6b3a7640000",
      "0x0",
      "0xSignature64Bytes...",
      "0xPublicKey32Bytes..."
    ],
    "id": 1
  }'
```

**Error Responses**:
- `-32001`: Invalid nonce (expected vs actual)
- `-32002`: Invalid signature
- `-32000`: Transfer failed (e.g., insufficient balance)
- `-32602`: Invalid params

---

### merklith_signAndSendTransaction

Signs a transaction with a private key and sends it in one step.

**Parameters**:
1. `STRING` - Private key (0x-prefixed, 32 bytes hex)
2. `STRING` - To address
3. `STRING` - Amount in wei (hex or decimal)

**Returns**: `Object` - Transaction result

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_signAndSendTransaction",
    "params": [
      "0xYourPrivateKey...",
      "0x8ba1f109551bD432803012645Hac136c82C3e82",
      "0xde0b6b3a7640000"
    ],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "txHash": "0xabc123...",
    "from": "0x742d35...",
    "signature": "0xsig..."
  },
  "id": 1
}
```

---

### merklith_sendRawTransaction

Submits a raw transaction (returns a random hash, placeholder implementation).

**Parameters**:
1. `STRING` - Raw transaction data

**Returns**: `STRING` - Transaction hash

---

### merklith_getBlockByNumber

Returns block information by number.

**Parameters**:
1. `STRING` - Block number as hex or `"latest"`

**Returns**: `Object|null` - Block object or null if not found

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getBlockByNumber",
    "params": ["latest"],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "number": "0x1",
    "hash": "0xabc123...",
    "parentHash": "0xdef456...",
    "nonce": "0x0000000000000000",
    "transactions": [],
    "gasLimit": "0x1c9c380",
    "gasUsed": "0x0",
    "timestamp": "0x65a4d800"
  },
  "id": 1
}
```

---

### merklith_getBlockInfo

Returns extended block information including transaction count.

**Parameters**:
1. `STRING` - Block number as hex or `"latest"`

**Returns**: `Object` - Extended block info

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getBlockInfo",
    "params": ["latest"],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "number": "0x1",
    "hash": "0xabc123...",
    "parentHash": "0xdef456...",
    "timestamp": "0x65a4d800",
    "txCount": 5
  },
  "id": 1
}
```

---

### merklith_getCurrentBlockHash

Returns the hash of the current block.

**Parameters**: None

**Returns**: `STRING` - Block hash (0x-prefixed)

---

### merklith_getBlockChain

Returns a range of blocks.

**Parameters**:
1. `NUMBER` - Start block number
2. `NUMBER` - Count (max 100)

**Returns**: `Array` - Array of block objects

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getBlockChain",
    "params": [0, 10],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": [
    {
      "number": "0x0",
      "hash": "0x...",
      "parentHash": "0x...",
      "timestamp": "0x..."
    }
  ],
  "id": 1
}
```

---

### merklith_getChainStats

Returns overall chain statistics.

**Parameters**: None

**Returns**: `Object` - Chain statistics

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getChainStats",
    "params": [],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "chainId": "0x4269",
    "blockNumber": "0x3039",
    "blockHash": "0xabc123...",
    "accounts": 42
  },
  "id": 1
}
```

---

### merklith_getTransactionByHash

Returns transaction details by hash (placeholder implementation).

**Parameters**:
1. `STRING` - Transaction hash

**Returns**: `Object` - Transaction object

---

### merklith_gasPrice

Returns the current gas price.

**Parameters**: None

**Returns**: `STRING` - Gas price in sparks (hex). Returns `0x3b9aca00` (1 gwei).

---

### merklith_estimateGas

Returns estimated gas for a transfer.

**Parameters**: None

**Returns**: `STRING` - Estimated gas (hex). Returns `0x5208` (21000).

---

### merklith_version

Returns the node version.

**Parameters**: None

**Returns**: `STRING` - Version string (`merklith/0.1.0`)

---

### merklith_syncing

Returns sync status.

**Parameters**: None

**Returns**: `BOOLEAN` - Always `false` (single-node mode)

---

### merklith_accounts

Returns all accounts with balances.

**Parameters**: None

**Returns**: `Array` - Array of address strings (0x-prefixed)

---

### merklith_createWallet

Creates a new Ed25519 keypair.

**Parameters**: None

**Returns**: `Object` - Wallet with address and private key

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

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "address": "0x742d35cc6634c0532925a3b844bc9e7595f0beb0",
    "privateKey": "0xabcdef1234567890..."
  },
  "id": 1
}
```

---

### merklith_deployContract

Deploys a contract.

**Parameters**:
1. `STRING` - Deployer address
2. `STRING` - Contract bytecode (0x-prefixed hex)

**Returns**: `STRING` - Contract address

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_deployContract",
    "params": [
      "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
      "0x6080604052..."
    ],
    "id": 1
  }'
```

---

### merklith_getCode

Returns bytecode at a given address.

**Parameters**:
1. `STRING` - Contract address

**Returns**: `STRING` - Bytecode (0x-prefixed hex)

---

### merklith_getStorageAt

Returns the value from a storage position at a given address.

**Parameters**:
1. `STRING` - Contract address
2. `STRING` - Storage key (0x-prefixed, 32 bytes hex)

**Returns**: `STRING` - Storage value (0x-prefixed, 32 bytes hex)

---

### merklith_call

Executes a call (read-only) against a contract.

**Parameters**:
1. `STRING` - Contract address
2. `STRING` - Call data (0x-prefixed hex)

**Returns**: `STRING` - Return data (0x-prefixed hex)

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_call",
    "params": [
      "0xContractAddress...",
      "0x70a08231000000000000000000000000742d35cc6634c0532925a3b844bc9e7595f0beb"
    ],
    "id": 1
  }'
```

---

### merklith_createAttestation

Creates a BLS attestation for a block (validator only).

**Parameters**:
1. `STRING` - Validator private key (0x-prefixed, 32 bytes hex)
2. `STRING` - Block number (hex or decimal)

**Returns**: `Object` - Attestation data

**Example**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_createAttestation",
    "params": ["0xValidatorPrivateKey...", "0x1"],
    "id": 1
  }'
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "blockNumber": "0x1",
    "blockHash": "0xabc123...",
    "attester": "0x742d35...",
    "signature": "0xblsSig..."
  },
  "id": 1
}
```

---

## Ethereum Compatibility Aliases

These aliases exist for compatibility with Ethereum tooling (e.g., MetaMask, web3.js, ethers.js).
Both `merklith_*` and `eth_*` versions work simultaneously.

### Chain/Node Info

| Alias | Maps To | Notes |
|-------|---------|-------|
| `eth_chainId` | `merklith_chainId` | Returns hex chain ID |
| `eth_blockNumber` | `merklith_blockNumber` | Returns hex block number |
| `eth_gasPrice` | `merklith_gasPrice` | Returns 1 gwei |
| `eth_estimateGas` | `merklith_estimateGas` | Returns 21000 |
| `eth_syncing` | `merklith_syncing` | Returns false |
| `eth_mining` | - | Returns true |
| `eth_hashrate` | - | Returns "0x0" |
| `eth_protocolVersion` | - | Returns "0x41" |
| `eth_coinbase` | - | Returns zero address |
| `eth_feeHistory` | - | Returns placeholder |
| `eth_maxPriorityFeePerGas` | - | Returns "0x0" |

### Account Methods

| Alias | Maps To | Params |
|-------|---------|--------|
| `eth_getBalance` | `merklith_getBalance` | [address, block_tag] |
| `eth_getTransactionCount` | `merklith_getNonce` | [address, block_tag] |
| `eth_accounts` | `merklith_accounts` | (none) |
| `eth_getCode` | `merklith_getCode` | [address, block_tag] |
| `eth_getStorageAt` | `merklith_getStorageAt` | [address, slot, block_tag] |

### Block Methods

| Alias | Maps To | Params |
|-------|---------|--------|
| `eth_getBlockByNumber` | `merklith_getBlockByNumber` | [block, full_txs] |
| `eth_getBlockByHash` | - | [hash, full_txs] (placeholder) |
| `eth_getBlockTransactionCountByHash` | - | [hash] |
| `eth_getBlockTransactionCountByNumber` | - | [block] |
| `eth_getUncleCountByBlockHash` | - | Returns "0x0" |
| `eth_getUncleCountByBlockNumber` | - | Returns "0x0" |

### Transaction Methods

| Alias | Maps To | Params |
|-------|---------|--------|
| `eth_sendTransaction` | `merklith_transfer` | [{from, to, value, ...}] |
| `eth_sendRawTransaction` | `merklith_sendRawTransaction` | [signed_data] |
| `eth_getTransactionByHash` | `merklith_getTransactionByHash` | [hash] |
| `eth_getTransactionReceipt` | - | [hash] |

### Contract Methods

| Alias | Maps To | Params |
|-------|---------|--------|
| `eth_call` | `merklith_call` | [{to, data, ...}, block_tag] |

### Web3/Net Methods

| Alias | Maps To | Notes |
|-------|---------|-------|
| `web3_clientVersion` | `merklith_version` | Returns `merklith/0.1.0` |
| `web3_sha3` | - | Blake3 hash of input |
| `net_version` | - | Returns chain ID as decimal string |
| `net_listening` | - | Returns true |
| `net_peerCount` | - | Returns "0x0" |

**Note**: `eth_*` methods accept standard Ethereum parameter formats (e.g., `block_tag` as second param). The `block_tag` parameter is accepted but currently ignored (always uses latest state).

**Example** (using `eth_sendTransaction`):
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_sendTransaction",
    "params": [{
      "from": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
      "to": "0x8ba1f109551bD432803012645Hac136c82C3e82",
      "value": "0xde0b6b3a7640000"
    }],
    "id": 1
  }'
```

---

## Data Types

### Quantities

Unsigned integers encoded as hexadecimal strings.

Examples:
- `"0x0"` = 0
- `"0x1"` = 1
- `"0x4269"` = 17001
- `"0xde0b6b3a7640000"` = 1000000000000000000 (1 ANV)

### Addresses

20-byte hex strings with 0x prefix.

Example: `"0x742d35cc6634c0532925a3b844bc9e7595f0beb0"`

### Hashes

32-byte hex strings with 0x prefix.

Example: `"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"`

---

## Examples

### Check Balance

```bash
ADDRESS="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"method\": \"merklith_getBalance\",
    \"params\": [\"$ADDRESS\"],
    \"id\": 1
  }"
```

### Transfer Funds

```bash
FROM="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
TO="0x8ba1f109551bD432803012645Hac136c82C3e82"
VALUE="0xde0b6b3a7640000"  # 1 ANV

curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"method\": \"merklith_transfer\",
    \"params\": [\"$FROM\", \"$TO\", \"$VALUE\"],
    \"id\": 1
  }"
```

### Get Latest Block

```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getBlockByNumber",
    "params": ["latest"],
    "id": 1
  }'
```

### Get Chain Stats

```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "merklith_getChainStats",
    "params": [],
    "id": 1
  }'
```

### Create Wallet

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

## Python Example

```python
import requests

def rpc_call(method, params=[]):
    response = requests.post('http://localhost:8545', json={
        'jsonrpc': '2.0',
        'method': method,
        'params': params,
        'id': 1
    })
    return response.json()['result']

# Get chain stats
stats = rpc_call('merklith_getChainStats')
print(f"Chain ID: {stats['chainId']}")
print(f"Block: {int(stats['blockNumber'], 16)}")

# Get balance
balance = rpc_call('merklith_getBalance', ['0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'])
print(f"Balance: {int(balance, 16)} wei")

# Get block
block = rpc_call('merklith_getBlockByNumber', ['latest'])
print(f"Block: {int(block['number'], 16)}")

# Create wallet
wallet = rpc_call('merklith_createWallet')
print(f"Address: {wallet['address']}")
```

## JavaScript Example

```javascript
const axios = require('axios');

async function rpcCall(method, params = []) {
  const response = await axios.post('http://localhost:8545', {
    jsonrpc: '2.0',
    method: method,
    params: params,
    id: 1
  });
  return response.data.result;
}

async function main() {
  // Get chain stats
  const stats = await rpcCall('merklith_getChainStats');
  console.log('Chain ID:', stats.chainId);

  // Get block number
  const blockNumber = await rpcCall('merklith_blockNumber');
  console.log('Block Number:', parseInt(blockNumber, 16));

  // Create wallet
  const wallet = await rpcCall('merklith_createWallet');
  console.log('New wallet:', wallet.address);

  // Transfer
  const txHash = await rpcCall('merklith_transfer', [
    '0xFromAddress...', '0xToAddress...', '0xde0b6b3a7640000'
  ]);
  console.log('TX Hash:', txHash);
}

main();
```

## Rate Limiting

Default rate limits:
- 100 requests per second per IP
- 1000 requests per minute per IP

Configure in node settings:
```toml
[rpc]
rate_limit = 1000  # requests per minute
```

## See Also

- [CLI Guide](CLI_GUIDE.md) - Command-line interface
- [Explorer Guide](EXPLORER.md) - TUI block explorer
- [Architecture](ARCHITECTURE.md) - System design
