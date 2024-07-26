use std::str::FromStr;

use ethers_core::{
    types::Eip1559TransactionRequest,
    utils::{
        keccak256,
        rlp::{Decodable, Rlp},
    },
};
// Find all our documentation at https://docs.near.org
use near_sdk::serde_json::json;
use near_sdk::{env, log, near, AccountId, Gas, NearToken, Promise};

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

        let mut vec = vec![u8::from(2)];
        vec.extend(tx.rlp().to_vec());
        log!("parsed tx: [{}] {:?}", vec.len(), vec);

        let mut payload = keccak256(vec);
        payload.reverse();

        log!("reverted payload: [{}] {:?}", payload.len(), payload);

        let args = json!({
            "request": {
                "payload": payload,
                "path": "ethereum-1",
                "key_version": 0
            }
        })
        .to_string()
        .into_bytes();
        Promise::new(AccountId::from_str("v1.signer-prod.testnet").unwrap()).function_call(
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
