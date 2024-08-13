use std::str::FromStr;

use ethers_core::types::{Bytes, Eip1559TransactionRequest, NameOrAddress, H160};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Timestamp};

pub type RequestId = u64;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct InputRequest {
    pub allowed_executors: Vec<Executor>,
    pub base_eip1559_payload: BaseEip1559TransactionPayload,
    pub derivation_seed_number: u32,
}

impl InputRequest {
    pub fn validate(&self) -> Option<&str> {
        if self.allowed_executors.len() == 0 {
            return Some("At least one executor must be provided");
        }

        if self.allowed_executors.len() > 30 {
            return Some("ERR_TOO_MANY_EXECUTORS");
        }

        None
    }
}

/// An internal request wrapped with predecessor of a request
#[derive(BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Request {
    pub id: RequestId,
    pub allowed_executors: Vec<Executor>,
    pub payload: BaseEip1559TransactionPayload,
    pub derivation_path: String,
    pub key_version: u32,
    pub deadline: Timestamp,
}

/// Represents executor of get_signature request: account
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde", untagged)]
#[borsh(crate = "near_sdk::borsh")]
pub enum Executor {
    Account { account_id: AccountId },
    // TODO: bring AccessKey { public_key: PublicKey }
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[serde(crate = "near_sdk::serde", rename_all = "snake_case")]
#[borsh(crate = "near_sdk::borsh")]
pub struct BaseEip1559TransactionPayload {
    pub to: String,
    pub data: Option<String>,
    pub value: Option<U128>,
    pub nonce: u128,
}

impl Into<Eip1559TransactionRequest> for BaseEip1559TransactionPayload {
    fn into(self) -> Eip1559TransactionRequest {
        let to = NameOrAddress::Address(H160::from_str(&self.to).expect("ERR_CANT_PARSE_ADDRESS"));
        let nonce = self.nonce;
        let value = self.value.unwrap_or(U128(0)).0;
        let data = Bytes::from_str(self.data.unwrap_or("0x".to_string()).as_str())
            .expect("ERR_CANT_PARSE_DATA");

        Eip1559TransactionRequest::new()
            .to(to)
            .nonce(nonce)
            .value(value)
            .data(data)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde", rename_all = "snake_case")]
pub struct OtherEip1559TransactionPayload {
    pub chain_id: u64,
    pub max_fee_per_gas: U128,
    pub max_priority_fee_per_gas: U128,
    pub gas: Option<U128>,
}

impl OtherEip1559TransactionPayload {
    pub fn include_into_base_tx(
        &self,
        base_tx: Eip1559TransactionRequest,
    ) -> Eip1559TransactionRequest {
        base_tx
            .clone()
            .chain_id(self.chain_id)
            .gas(self.gas.unwrap_or(U128(21_000)).0)
            .max_fee_per_gas(self.max_fee_per_gas.0)
            .max_priority_fee_per_gas(self.max_priority_fee_per_gas.0)
    }
}

impl Into<Eip1559TransactionRequest> for OtherEip1559TransactionPayload {
    fn into(self) -> Eip1559TransactionRequest {
        Eip1559TransactionRequest::new().chain_id(5)
    }
}

#[cfg(test)]
mod tests {
    use ethers_core::types::Eip1559TransactionRequest;
    use near_sdk::json_types::U128;

    use crate::BaseEip1559TransactionPayload;

    use super::OtherEip1559TransactionPayload;

    fn base_payload() -> BaseEip1559TransactionPayload {
        BaseEip1559TransactionPayload {
            to: "0x0000000000000000000000000000000000000000".to_string(),
            nonce: 0,
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
    fn test_into_eip1559_tx() {
        let payload = base_payload();
        let _: Eip1559TransactionRequest = payload.into();
    }

    #[should_panic = "ERR_CANT_PARSE_ADDRESS"]
    #[test]
    fn test_into_eip1559_tx_panics_on_empty_address() {
        let mut payload = base_payload();
        payload.to = "".to_string();

        let _: Eip1559TransactionRequest = payload.into();
    }

    #[should_panic = "ERR_CANT_PARSE_ADDRESS"]
    #[test]
    fn test_into_eip1559_tx_panics_on_wrong_address() {
        let mut payload = base_payload();
        payload.to = "4141ajkl412pp41fakfa".to_string();

        let _: Eip1559TransactionRequest = payload.into();
    }

    #[should_panic = "ERR_CANT_PARSE_DATA"]
    #[test]
    fn test_into_eip1559_tx_panics_on_wrong_data() {
        let mut payload = base_payload();
        payload.data = Some("4141ajkl412pp41fakfa".to_string());

        let _: Eip1559TransactionRequest = payload.into();
    }

    #[test]
    fn test_building_eip1559_tx() {
        let base_payload = base_payload();
        let base_tx: Eip1559TransactionRequest = base_payload.into();

        let other_payload = other_payload();
        let tx = other_payload.include_into_base_tx(base_tx);

        assert_eq!(
            tx.nonce.clone(),
            Some(ethers_core::types::U256([0, 0, 0, 0]))
        );
        assert_eq!(tx.chain_id, Some(ethers_core::types::U64([1111])));
    }

    // TODO: tests for OtherEip1559TransactionPayload
}
