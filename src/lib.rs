mod primitives;

use ethers_core::{types::Eip1559TransactionRequest, utils::keccak256};

use near_sdk::serde_json::json;
use near_sdk::{
    env::{self, block_timestamp},
    log, near, require, AccountId, Duration, Gas, PanicOnDefault, Promise,
};
use near_sdk::{store::LookupMap, NearToken};
use primitives::*;

const ONE_MINUTE_NANOS: Duration = 60_000_000_000;
const MIN_GAS_FOR_MPC_SIGN: Gas = Gas::from_tgas(250);
const GAS_FOR_BASIC_OP: Gas = Gas::from_tgas(5);
const GAS_FOR_PROMISE: Gas = Gas::from_tgas(5);

fn generate_derivation_path(predecessor_id: AccountId, seed_number: u32) -> String {
    format!("{}-{}", predecessor_id, seed_number)
}

fn get_request_executor(request: &Request) -> Option<Executor> {
    let predecessor_id = env::predecessor_account_id();

    request
        .allowed_executors
        .iter()
        .find(|&executor| match executor {
            Executor::Account { account_id } => predecessor_id == *account_id,
        })
        .cloned()
}

fn min_required_gas() -> Gas {
    MIN_GAS_FOR_MPC_SIGN
        .checked_add(GAS_FOR_BASIC_OP)
        .unwrap()
        .checked_add(GAS_FOR_PROMISE)
        .unwrap()
}

// Define the contract structure
#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    /// Last available id for the requests.
    pub last_request_id: RequestId,
    /// Map of signing requests
    pub requests: LookupMap<RequestId, Request>,
    /// MPC Account ID
    pub signer_account_id: AccountId,
}

// Implement the contract structure
#[near]
impl Contract {
    #[init]
    pub fn new(signer_account_id: AccountId) -> Self {
        Self {
            last_request_id: 0,
            requests: LookupMap::new(b"r"),
            signer_account_id: signer_account_id.clone(),
        }
    }

    pub fn set_signer_account_id(&mut self, account_id: AccountId) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "ERR_FORBIDDEN_ONLY_OWNER"
        );

        self.signer_account_id = account_id;
    }

    pub fn get_signer_account_id(&self) -> AccountId {
        self.signer_account_id.clone()
    }

    #[payable]
    pub fn register_signature_request(&mut self, request: InputRequest) -> RequestId {
        if let Some(validation_error) = request.validate() {
            panic!("{}", validation_error);
        }

        let storage_used_before = env::storage_usage();

        let current_request_id = self.last_request_id;
        self.last_request_id += 1;

        let internal_request = Request {
            id: current_request_id,
            allowed_executors: request.allowed_executors,
            payload: request.base_eip1559_payload,
            derivation_path: generate_derivation_path(
                env::predecessor_account_id(),
                request.derivation_seed_number,
            ),
            key_version: 0,
            deadline: block_timestamp() + 15 * ONE_MINUTE_NANOS,
        };
        self.requests.insert(internal_request.id, internal_request);

        let storage_used_after = env::storage_usage();

        let used_storage = (storage_used_after - storage_used_before) as u128;

        let storage_deposit = env::storage_byte_cost()
            .checked_mul(used_storage)
            .expect("ERR_STORAGE_DEPOSIT_CALC");

        require!(
            env::attached_deposit() >= storage_deposit,
            format!(
                "Deposited amount must be bigger than {} yocto",
                storage_deposit
            )
        );

        let refund = env::attached_deposit()
            .checked_sub(storage_deposit)
            .unwrap();
        if refund > NearToken::from_yoctonear(1) {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }

        current_request_id
    }

    #[payable]
    pub fn get_signature(
        &mut self,
        request_id: RequestId,
        other_payload: OtherEip1559TransactionPayload,
    ) -> Promise {
        require!(
            env::prepaid_gas() >= min_required_gas(),
            "ERR_INSUFFICIENT_GAS"
        );

        // TODO: use errors from Enum
        let request = self.requests.get(&request_id).expect("ERR_NOT_FOUND");

        require!(env::block_timestamp() <= request.deadline, "ERR_TIME_IS_UP");

        get_request_executor(&request).expect("ERR_FORBIDDEN");

        let base_tx: Eip1559TransactionRequest = request.payload.clone().into();
        let tx = other_payload.include_into_base_tx(base_tx);

        log!(
            "Requesting signature to {:?} contract with nonce {:?} and attached {:?} wETH",
            tx.to.clone().unwrap(),
            tx.nonce.unwrap(),
            tx.value.unwrap()
        );

        let mut vec = vec![u8::from(2)];
        vec.extend(tx.rlp().to_vec());
        log!("parsed tx: [{}] {:?}", vec.len(), vec);

        let payload = keccak256(vec);
        log!("payload: [{}] {:?}", payload.len(), payload);

        let args = json!({
            "request": {
                "payload": payload,
                "path": request.derivation_path,
                "key_version": request.key_version
            }
        })
        .to_string()
        .into_bytes();

        let gas_for_sign_promise = env::prepaid_gas()
            .checked_sub(env::used_gas())
            .unwrap()
            // some Gas is gonna be used to create Promise
            .checked_sub(GAS_FOR_PROMISE)
            .unwrap();

        Promise::new(self.signer_account_id.clone()).function_call(
            "sign".to_owned(),
            args,
            env::attached_deposit(),
            gas_for_sign_promise,
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
