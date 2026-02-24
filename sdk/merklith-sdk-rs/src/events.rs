//! Event handling and subscriptions.

use merklith_types::Address;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::errors::Result;
use crate::types::{Filter, Log};

/// Event subscription stream.
pub struct EventStream {
    _marker: std::marker::PhantomData<Log>,
}

impl EventStream {
    /// Create new event stream.
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl Default for EventStream {
    fn default() -> Self {
        Self::new()
    }
}

/// Event decoder trait.
pub trait EventDecoder {
    /// Decode event from log.
    fn decode(log: &Log) -> Result<Self>
    where
        Self: Sized;
}

/// Event encoder trait.
pub trait EventEncoder {
    /// Encode event to topics and data.
    fn encode(&self) -> (Vec<[u8; 32]>, Vec<u8>);
}

/// Parse event signature to topic.
pub fn event_signature_to_topic(signature: &str) -> [u8; 32] {
    let hash = blake3::hash(signature.as_bytes());
    let mut topic = [0u8; 32];
    topic.copy_from_slice(hash.as_bytes());
    topic
}

/// Create event filter for specific event.
pub fn event_filter(
    contract: Address,
    event_signature: &str,
) -> Filter {
    let topic = event_signature_to_topic(event_signature);
    
    Filter::new()
        .address(contract)
        .topic(Some(vec![topic]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_signature_to_topic() {
        let topic = event_signature_to_topic("Transfer(address,address,uint256)");
        assert_ne!(topic, [0u8; 32]);
    }

    #[test]
    fn test_event_filter() {
        let filter = event_filter(Address::ZERO, "Transfer(address,address,uint256)");
        
        assert_eq!(filter.addresses.len(), 1);
        assert_eq!(filter.topics.len(), 1);
    }
}
