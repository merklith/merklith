//! MERKLITH Smart Contract Examples
//! 
//! Production-ready smart contract examples for the MERKLITH blockchain.

pub mod erc20;
pub mod erc721;
pub mod bridge;
pub mod governance;

pub use erc20::{ERC20Token, TransferEvent, ApprovalEvent, ERC20Error};
pub use erc721::{ERC721Token, TransferEvent as NFTTransferEvent, ApprovalEvent as NFTApprovalEvent, ERC721Error};
pub use bridge::{BridgeContract, BridgeEvent, BridgeRequest, BridgeError};
pub use governance::{GovernanceContract, Proposal, ProposalEvent, VoteEvent, GovernanceError};

/// Contract version
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_version() {
        assert_eq!(CONTRACT_VERSION, "0.1.0");
    }
}
