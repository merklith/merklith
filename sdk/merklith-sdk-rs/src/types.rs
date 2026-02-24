//! SDK types and utilities.

use merklith_types::{Address, U256};

/// Block identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockId {
    /// Block number
    Number(u64),
    /// Block hash
    Hash([u8; 32]),
    /// Latest block
    Latest,
    /// Pending block
    Pending,
    /// Safe block
    Safe,
    /// Finalized block
    Finalized,
    /// Earliest block (genesis)
    Earliest,
}

impl BlockId {
    /// Convert to RPC parameter string.
    pub fn to_string(&self) -> String {
        match self {
            BlockId::Number(n) => format!("0x{:x}", n),
            BlockId::Hash(h) => format!("0x{}", hex::encode(h)),
            BlockId::Latest => "latest".to_string(),
            BlockId::Pending => "pending".to_string(),
            BlockId::Safe => "safe".to_string(),
            BlockId::Finalized => "finalized".to_string(),
            BlockId::Earliest => "earliest".to_string(),
        }
    }
}

impl Default for BlockId {
    fn default() -> Self {
        BlockId::Latest
    }
}

/// Transaction options.
#[derive(Debug, Clone, Default)]
pub struct TxOptions {
    /// From address (if not using default)
    pub from: Option<Address>,
    /// Gas price
    pub gas_price: Option<U256>,
    /// Max fee per gas (EIP-1559)
    pub max_fee_per_gas: Option<U256>,
    /// Max priority fee (EIP-1559)
    pub max_priority_fee: Option<U256>,
    /// Gas limit
    pub gas_limit: Option<u64>,
    /// Value
    pub value: Option<U256>,
    /// Nonce
    pub nonce: Option<u64>,
}

impl TxOptions {
    /// Create new transaction options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set from address.
    pub fn from(mut self, addr: Address) -> Self {
        self.from = Some(addr);
        self
    }

    /// Set gas price.
    pub fn gas_price(mut self, price: U256) -> Self {
        self.gas_price = Some(price);
        self
    }

    /// Set gas limit.
    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    /// Set value.
    pub fn value(mut self, value: U256) -> Self {
        self.value = Some(value);
        self
    }

    /// Set nonce.
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }
}

/// Contract call options.
#[derive(Debug, Clone, Default)]
pub struct CallOptions {
    /// Block to query at
    pub block: BlockId,
    /// Gas limit
    pub gas_limit: Option<u64>,
    /// Gas price
    pub gas_price: Option<U256>,
    /// Value
    pub value: Option<U256>,
}

impl CallOptions {
    /// Create new call options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set block.
    pub fn at_block(mut self, block: BlockId) -> Self {
        self.block = block;
        self
    }
}

/// Account info.
#[derive(Debug, Clone)]
pub struct AccountInfo {
    /// Address
    pub address: Address,
    /// Balance
    pub balance: U256,
    /// Nonce
    pub nonce: u64,
    /// Code hash (None for EOA)
    pub code_hash: Option<[u8; 32]>,
}

/// Transaction receipt.
#[derive(Debug, Clone)]
pub struct TransactionReceipt {
    /// Transaction hash
    pub transaction_hash: [u8; 32],
    /// Transaction index
    pub transaction_index: u64,
    /// Block hash
    pub block_hash: [u8; 32],
    /// Block number
    pub block_number: u64,
    /// From
    pub from: Address,
    /// To
    pub to: Option<Address>,
    /// Gas used
    pub gas_used: u64,
    /// Status (1 = success, 0 = failure)
    pub status: u8,
    /// Logs
    pub logs: Vec<Log>,
}

/// Log entry.
#[derive(Debug, Clone)]
pub struct Log {
    /// Address
    pub address: Address,
    /// Topics
    pub topics: Vec<[u8; 32]>,
    /// Data
    pub data: Vec<u8>,
    /// Block number
    pub block_number: u64,
    /// Transaction hash
    pub transaction_hash: [u8; 32],
    /// Log index
    pub log_index: u64,
}

/// Filter for event logs.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// From block
    pub from_block: Option<BlockId>,
    /// To block
    pub to_block: Option<BlockId>,
    /// Addresses
    pub addresses: Vec<Address>,
    /// Topic filters
    pub topics: Vec<Option<Vec<[u8; 32]>>>,
}

impl Filter {
    /// Create new filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set from block.
    pub fn from_block(mut self, block: BlockId) -> Self {
        self.from_block = Some(block);
        self
    }

    /// Set to block.
    pub fn to_block(mut self, block: BlockId) -> Self {
        self.to_block = Some(block);
        self
    }

    /// Add address.
    pub fn address(mut self, addr: Address) -> Self {
        self.addresses.push(addr);
        self
    }

    /// Add topic filter.
    pub fn topic(mut self, topic: Option<Vec<[u8; 32]>>) -> Self {
        self.topics.push(topic);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_id() {
        assert_eq!(BlockId::Number(100).to_string(), "0x64");
        assert_eq!(BlockId::Latest.to_string(), "latest");
        assert_eq!(BlockId::Pending.to_string(), "pending");
    }

    #[test]
    fn test_tx_options() {
        let opts = TxOptions::new()
            .gas_limit(100_000)
            .value(U256::from(1000u64));

        assert_eq!(opts.gas_limit, Some(100_000));
        assert_eq!(opts.value, Some(U256::from(1000u64)));
    }

    #[test]
    fn test_filter() {
        let addr = Address::ZERO;
        let filter = Filter::new()
            .from_block(BlockId::Number(0))
            .to_block(BlockId::Latest)
            .address(addr);

        assert!(filter.from_block.is_some());
        assert!(filter.to_block.is_some());
        assert_eq!(filter.addresses.len(), 1);
    }
}
