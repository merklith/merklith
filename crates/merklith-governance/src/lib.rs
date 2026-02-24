//! Merklith Governance - Liquid democracy governance system.
//!
//! This crate provides:
//! - Proposal lifecycle management
//! - Quadratic voting with time-lock
//! - Liquid democracy delegation
//! - Treasury management

pub mod proposal;
pub mod voting;
pub mod delegation;
pub mod treasury;
pub mod error;

pub use proposal::{Proposal, ProposalType};
pub use voting::{calculate_voting_power, LockDuration};
pub use delegation::{DelegationGraph, resolve_voting_power};
pub use error::GovernanceError;
