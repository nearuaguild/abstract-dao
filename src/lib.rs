use std::str::FromStr;

use ethers_core::{
    types::{Eip1559TransactionRequest, NameOrAddress, H160, U64},
    utils::{
        keccak256,
        rlp::{Decodable, Rlp},
    },
};
// Find all our documentation at https://docs.near.org
use near_sdk::{
    env::{self},
    json_types::U128,
    log, near, AccountId, Gas, NearToken, Promise,
};
use near_sdk::{
    serde::{Deserialize, Serialize},
    serde_json::json,
};

// Define the contract structure
#[near(contract_state)]
pub struct Contract {
    greeting: String,
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            greeting: "Hello".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde", rename_all = "snake_case")]
pub struct EthereumPayload {
    pub chain_id: u64,
    pub to: Vec<u8>,
    pub data: Option<Vec<u8>>,
    pub value: Option<U128>,
    pub nonce: u128,
    pub max_fee_per_gas: U128,
    pub max_priority_fee_per_gas: U128,
    pub gas: Option<U128>,
}

// Implement the contract structure
#[near]
impl Contract {
    // Public method - returns the greeting saved, defaulting to DEFAULT_GREETING
    pub fn get_greeting(&self) -> String {
        self.greeting.clone()
    }

    // Public method - accepts a greeting, such as "howdy", and records it
    pub fn set_greeting(&mut self, greeting: String) {
        log!("Saving greeting: {greeting}");
        self.greeting = greeting;
    }

    pub fn reconstruct_payload(&self, bytes: Vec<u8>) -> Promise {
        let rlp = Rlp::new(bytes.as_slice());
        let result = Eip1559TransactionRequest::decode(&rlp);

        let tx = match result {
            Ok(tx) => tx,
            Err(error) => env::panic_str(format!("Couldn't decode - {}", error).as_str()),
        };

        log!("tx receiver: {:?}", tx.to);
        log!(
            "Requesting signature to {:?} contract with nonce {:?} and attached {:?} wETH",
            tx.to,
            tx.nonce,
            tx.value
        );

        let mut vec = vec![u8::from(2)];
        vec.extend(tx.rlp().to_vec());
        log!("parsed tx: [{}] {:?}", vec.len(), vec);

        let payload = keccak256(vec);

        log!("payload: [{}] {:?}", payload.len(), payload);

        let args = json!({
            "request": {
                "payload": payload,
                "path": "ethereum-1",
                "key_version": 0
            }
        })
        .to_string()
        .into_bytes();
        Promise::new(AccountId::from_str("v1.signer-dev.testnet").unwrap()).function_call(
            "sign".to_owned(),
            args,
            NearToken::from_yoctonear(100000000000000000000000),
            Gas::from_tgas(290),
        )
    }

    pub fn get_signature(&self, payload: EthereumPayload, derivation_path: String) -> Promise {
        log!("Started!");
        let tx = Eip1559TransactionRequest::new()
            .chain_id(U64::from(payload.chain_id))
            .to(NameOrAddress::Address(H160::from_slice(
                payload.to.as_slice(),
            )))
            .nonce(payload.nonce)
            .value(payload.value.unwrap_or(U128(0)).0)
            .data(payload.data.unwrap_or(Vec::new()))
            .gas(payload.gas.unwrap_or(U128(21_000)).0)
            .max_fee_per_gas(payload.max_fee_per_gas.0)
            .max_priority_fee_per_gas(payload.max_priority_fee_per_gas.0);

        log!(
            "Requesting signature to {:?} contract with nonce {:?} and attached {:?} wETH",
            tx.to,
            tx.nonce,
            tx.value
        );

        let mut vec = vec![u8::from(2)];
        vec.extend(tx.rlp().to_vec());
        log!("parsed tx: [{}] {:?}", vec.len(), vec);

        let payload = keccak256(vec);
        log!("payload: [{}] {:?}", payload.len(), payload);

        let args = json!({
            "request": {
                "payload": payload,
                "path": derivation_path,
                "key_version": 0
            }
        })
        .to_string()
        .into_bytes();

        Promise::new(AccountId::from_str("v1.signer-dev.testnet").unwrap()).function_call(
            "sign".to_owned(),
            args,
            NearToken::from_yoctonear(1),
            Gas::from_tgas(290),
        )
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_default_greeting() {
        let contract = Contract::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(contract.get_greeting(), "Hello");
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(contract.get_greeting(), "howdy");
    }
}
