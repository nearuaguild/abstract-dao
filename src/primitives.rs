use std::str::FromStr;

use ethers_contract::encode_function_data;
use ethers_core::abi::{Function, Token, Tokenize};
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
#[near_sdk::near(serializers = [json])]
pub struct FunctionData {
    pub function_abi: Function,
    pub arguments: Vec<Token>,
}

impl Tokenize for FunctionData {
    fn into_tokens(self) -> Vec<Token> {
        self.arguments
    }
}

impl FunctionData {
    pub fn encode(&self) -> Bytes {
        encode_function_data(&self.function_abi, self.clone())
            .expect("Function arguments don't match provided ABI")
    }
}

#[derive(Clone)]
#[near_sdk::near(serializers = [json])]
pub struct InputTransactionPayload {
    pub function_data: Option<FunctionData>,
    pub to: String,
    pub value: Option<U128>,
    pub nonce: U128,
}

impl From<InputTransactionPayload> for BaseEip1559TransactionPayload {
    fn from(input: InputTransactionPayload) -> Self {
        let valid_address = H160::from_str(&input.to).expect("ERR_CANT_PARSE_ADDRESS");

        Self {
            to: Bytes::from(valid_address.0).to_string(),
            nonce: input.nonce,
            value: input.value,
            data: match input.function_data {
                Some(data) => Some(data.encode().to_string()),
                None => None,
            },
        }
    }
}

#[derive(Clone)]
#[near_sdk::near(serializers = [json])]
pub struct InputRequest {
    pub allowed_account_id: AccountId,
    pub transaction_payload: InputTransactionPayload,
    pub derivation_seed_number: u32,
    pub key_version: Option<u32>,
}

/// An internal request wrapped with predecessor of a request
#[derive(Clone)]
#[near_sdk::near(serializers = [borsh, json])]
pub struct Request {
    pub id: RequestId,
    pub allowed_account_id: AccountId,
    pub payload: BaseEip1559TransactionPayload,
    pub derivation_path: String,
    pub key_version: u32,
    pub deadline: Timestamp,
}

impl Request {
    pub fn is_time_exceeded(&self, now: Timestamp) -> bool {
        now > self.deadline
    }

    pub fn is_account_allowed(&self, account: AccountId) -> bool {
        self.allowed_account_id == account
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
    use ethers_core::{
        abi::{Param, ParamType, StateMutability},
        types::{Eip1559TransactionRequest, U256},
    };
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

    fn function_data(arguments: Vec<Token>) -> FunctionData {
        FunctionData {
            function_abi: Function {
                name: "set".to_string(),
                inputs: vec![Param {
                    name: "_num".to_string(),
                    kind: ParamType::Uint(256),
                    internal_type: Some("uint256".to_string()),
                }],
                outputs: vec![],
                constant: None,
                state_mutability: StateMutability::NonPayable,
            },
            arguments: arguments,
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

    #[test]
    fn test_input_transaction_payload_into_base_payload() {
        let input = InputTransactionPayload {
            to: "0x0000000000000000000000000000000000000000".to_string(),
            nonce: U128(0),
            value: Some(U128(1000)),
            function_data: Some(function_data(vec![Token::Uint(U256([2000, 0, 0, 0]))])),
        };

        let base_payload: BaseEip1559TransactionPayload = input.into();

        assert_eq!(
            base_payload.to,
            "0x0000000000000000000000000000000000000000".to_string()
        );
        assert_eq!(base_payload.nonce, U128(0));
        assert_eq!(base_payload.value, Some(U128(1000)));
        assert_eq!(
            base_payload.data,
            Some(
                "0x60fe47b100000000000000000000000000000000000000000000000000000000000007d0"
                    .to_string()
            )
        );
    }

    #[should_panic = "ERR_CANT_PARSE_ADDRESS"]
    #[test]
    fn test_input_transaction_payload_into_base_payload_panics_on_wrong_address() {
        let input = InputTransactionPayload {
            to: "0x0000000000000000000000000000000000000000?".to_string(),
            nonce: U128(0),
            value: Some(U128(1000)),
            function_data: None,
        };

        let _: BaseEip1559TransactionPayload = input.into();
    }

    #[should_panic = "Function arguments don't match provided ABI"]
    #[test]
    fn test_input_transaction_payload_into_base_payload_panics_on_invalid_function_arguments() {
        let input = InputTransactionPayload {
            to: "0x0000000000000000000000000000000000000000".to_string(),
            nonce: U128(0),
            value: Some(U128(1000)),
            function_data: Some(function_data(vec![])),
        };

        // must panic since one argument is expected, but wasn't provided
        let _: BaseEip1559TransactionPayload = input.into();
    }
}
