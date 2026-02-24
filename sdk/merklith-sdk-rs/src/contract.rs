//! Contract interaction helpers.

use merklith_types::{Address, Transaction, U256};
use std::marker::PhantomData;

use crate::client::Client;
use crate::errors::{Result, SdkError};
use crate::types::{CallOptions, TxOptions};

/// Contract interface.
pub struct Contract {
    client: Client,
    address: Address,
    abi: Option<serde_json::Value>,
}

impl Contract {
    /// Create new contract interface.
    pub fn new(client: Client, address: Address) -> Self {
        Self {
            client,
            address,
            abi: None,
        }
    }

    /// Create with ABI.
    pub fn with_abi(client: Client, address: Address, abi: serde_json::Value) -> Self {
        Self {
            client,
            address,
            abi: Some(abi),
        }
    }

    /// Get contract address.
    pub fn address(&self) -> Address {
        self.address
    }

    /// Call a contract method (read-only).
    pub async fn call(
        &self,
        data: Vec<u8>,
        options: CallOptions,
    ) -> Result<Vec<u8>> {
        let tx = Transaction::new(
            1337, // Default chain ID
            0,
            Some(self.address),
            options.value.unwrap_or(U256::ZERO),
            options.gas_limit.unwrap_or(100_000),
            U256::from(1_000_000_000u64),
            U256::from(1_000_000_000u64),
        );

        self.client.call(&tx, Some(options.block)).await
    }

    /// Send a transaction to contract.
    pub async fn send(
        &self,
        data: Vec<u8>,
        options: TxOptions,
    ) -> Result<merklith_types::Hash> {
        let tx = Transaction::new(
            1337, // Default chain ID
            options.nonce.unwrap_or(0),
            Some(self.address),
            options.value.unwrap_or(U256::ZERO),
            options.gas_limit.unwrap_or(100_000),
            options.gas_price.unwrap_or(U256::from(1_000_000_000u64)),
            U256::from(1_000_000_000u64),
        );

        self.client.send_transaction(&tx).await
    }

    /// Get contract bytecode.
    pub async fn code(&self) -> Result<Vec<u8>> {
        self.client.get_code(&self.address).await
    }

    /// Check if contract exists.
    pub async fn exists(&self) -> Result<bool> {
        let code = self.code().await?;
        Ok(!code.is_empty())
    }
}

/// Contract builder for deployment.
pub struct ContractBuilder {
    bytecode: Vec<u8>,
    abi: Option<serde_json::Value>,
    args: Vec<u8>,
}

impl ContractBuilder {
    /// Create new builder with bytecode.
    pub fn new(bytecode: Vec<u8>) -> Self {
        Self {
            bytecode,
            abi: None,
            args: vec![],
        }
    }

    /// Set ABI.
    pub fn with_abi(mut self, abi: serde_json::Value) -> Self {
        self.abi = Some(abi);
        self
    }

    /// Set constructor arguments.
    pub fn with_args(mut self, args: Vec<u8>) -> Self {
        self.args = args;
        self
    }

    /// Deploy the contract.
    pub async fn deploy(
        self,
        client: &Client,
        options: TxOptions,
    ) -> Result<Contract> {
        // Combine bytecode with constructor args
        let mut data = self.bytecode;
        data.extend_from_slice(&self.args);

        let tx = Transaction::new(
            1337, // Default chain ID
            options.nonce.unwrap_or(0),
            None, // Contract creation
            options.value.unwrap_or(U256::ZERO),
            options.gas_limit.unwrap_or(1_000_000),
            options.gas_price.unwrap_or(U256::from(1_000_000_000u64)),
            U256::from(1_000_000_000u64),
        );

        // Send transaction
        let hash = client.send_transaction(&tx).await?;

        // Wait for receipt to get contract address
        let receipt = client.wait_for_transaction(
            &hash,
            std::time::Duration::from_secs(60),
        ).await?;

        // Get contract address from receipt
        // For now, return a placeholder
        let contract_address = Address::ZERO;

        Ok(Contract {
            client: client.clone(),
            address: contract_address,
            abi: self.abi,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_creation() {
        let client = Client::new("http://localhost:8545");
        let contract = Contract::new(client, Address::ZERO);
        
        assert_eq!(contract.address(), Address::ZERO);
    }

    #[test]
    fn test_contract_builder() {
        let builder = ContractBuilder::new(vec![0x60, 0x80, 0x60])
            .with_args(vec![0x01, 0x02]);

        // Just ensure it compiles
        assert!(!builder.bytecode.is_empty());
    }
}
