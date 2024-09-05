use std::str::FromStr;

use ethers_core::types::{Bytes, Eip1559TransactionRequest, NameOrAddress, H160};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{AccountId, BorshStorageKey, Timestamp};

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    AllRequests,
}

pub type RequestId = u64;

#[derive(Clone)]
#[near_sdk::near(serializers = [borsh, json])]
pub struct InputRequest {
    pub allowed_actors: Vec<Actor>,
    pub base_eip1559_payload: BaseEip1559TransactionPayload,
    pub derivation_seed_number: u32,
    pub key_version: Option<u32>,
}

impl InputRequest {
    pub fn validate(&self) -> Option<&str> {
        if self.allowed_actors.is_empty() {
            return Some("At least one actor must be provided");
        }

        if self.allowed_actors.len() > 10 {
            return Some("ERR_TOO_MANY_ACTORS");
        }

        None
    }
}

/// An internal request wrapped with predecessor of a request
#[derive(Clone)]
#[near_sdk::near(serializers = [borsh, json])]
pub struct Request {
    pub id: RequestId,
    pub allowed_actors: Vec<Actor>,
    pub payload: BaseEip1559TransactionPayload,
    pub derivation_path: String,
    pub key_version: u32,
    pub deadline: Timestamp,
}

impl Request {
    pub fn is_time_exceeded(&self, now: Timestamp) -> bool {
        now > self.deadline
    }

    pub fn is_actor_allowed(&self, actor: Actor) -> bool {
        self.allowed_actors
            .iter()
            .any(|allowed_actor| *allowed_actor == actor)
    }
}

/// Represents entity that may have access to get_signature fn
#[derive(Clone, PartialEq)]
#[near_sdk::near(serializers = [borsh, json])]
pub enum Actor {
    Account { account_id: AccountId },
    // TODO: bring AccessKey { public_key: PublicKey }
}

impl From<AccountId> for Actor {
    fn from(account: AccountId) -> Self {
        Self::Account {
            account_id: account,
        }
    }
}

#[derive(Clone)]
#[near_sdk::near(serializers = [borsh, json])]
pub struct BaseEip1559TransactionPayload {
    pub to: String,
    pub data: Option<String>,
    // TODO: migrate to U256
    pub value: Option<U128>,
    // TODO: migrate to U256
    pub nonce: U128,
}

impl From<BaseEip1559TransactionPayload> for Eip1559TransactionRequest {
    fn from(payload: BaseEip1559TransactionPayload) -> Self {
        let to =
            NameOrAddress::Address(H160::from_str(&payload.to).expect("ERR_CANT_PARSE_ADDRESS"));
        let nonce = payload.nonce.0;
        let value = payload.value.unwrap_or(U128(0)).0;
        let data = Bytes::from_str(payload.data.unwrap_or("0x".to_string()).as_str())
            .expect("ERR_CANT_PARSE_DATA");

        Self::new().to(to).nonce(nonce).value(value).data(data)
    }
}

#[near_sdk::near(serializers = [borsh, json])]
pub struct OtherEip1559TransactionPayload {
    pub chain_id: u64,
    pub max_fee_per_gas: U128,
    pub max_priority_fee_per_gas: U128,
    pub gas: Option<U128>,
}

impl From<OtherEip1559TransactionPayload> for Eip1559TransactionRequest {
    fn from(payload: OtherEip1559TransactionPayload) -> Self {
        let chain_id = payload.chain_id;
        let gas = payload.gas.unwrap_or(U128(21_000)).0;
        let max_fee_per_gas = payload.max_fee_per_gas.0;
        let max_priority_fee_per_gas = payload.max_priority_fee_per_gas.0;

        Self::new()
            .chain_id(chain_id)
            .gas(gas)
            .max_fee_per_gas(max_fee_per_gas)
            .max_priority_fee_per_gas(max_priority_fee_per_gas)
    }
}

#[cfg(test)]
mod tests {
    use ethers_core::types::Eip1559TransactionRequest;
    use near_sdk::json_types::U128;

    use super::*;

    fn base_payload() -> BaseEip1559TransactionPayload {
        BaseEip1559TransactionPayload {
            to: "0x0000000000000000000000000000000000000000".to_string(),
            nonce: U128(0),
            value: Some(U128(1)),
            data: Some("0x2386f26fc10000".to_string()),
        }
    }

    fn other_payload() -> OtherEip1559TransactionPayload {
        OtherEip1559TransactionPayload {
            chain_id: 1111,
            gas: Some(U128(42_000)),
            max_fee_per_gas: U128(120_000),
            max_priority_fee_per_gas: U128(120_000),
        }
    }

    #[test]
    fn test_base_payload_into_eip1559_tx() {
        let payload = base_payload();
        let _: Eip1559TransactionRequest = payload.into();
    }

    #[should_panic = "ERR_CANT_PARSE_ADDRESS"]
    #[test]
    fn test_base_payload_into_eip1559_tx_panics_on_empty_address() {
        let mut payload = base_payload();
        payload.to = "".to_string();

        let _: Eip1559TransactionRequest = payload.into();
    }

    #[should_panic = "ERR_CANT_PARSE_ADDRESS"]
    #[test]
    fn test_base_payload_into_eip1559_tx_panics_on_wrong_address() {
        let mut payload = base_payload();
        payload.to = "4141ajkl412pp41fakfa".to_string();

        let _: Eip1559TransactionRequest = payload.into();
    }

    #[should_panic = "ERR_CANT_PARSE_DATA"]
    #[test]
    fn test_base_payload_into_eip1559_tx_panics_on_wrong_data() {
        let mut payload = base_payload();
        payload.data = Some("4141ajkl412pp41fakfa".to_string());

        let _: Eip1559TransactionRequest = payload.into();
    }

    #[test]
    fn test_other_payload_into_eip1559_tx() {
        let payload = other_payload();
        let _: Eip1559TransactionRequest = payload.into();
    }
}
