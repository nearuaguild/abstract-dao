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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use near_sdk::{json_types::U128, test_utils::VMContextBuilder, testing_env};
    // use primitives::{BaseEip1559TransactionPayload, InputRequest, OtherEip1559TransactionPayload};

    fn current() -> AccountId {
        AccountId::from_str("current").unwrap()
    }

    fn user1() -> AccountId {
        AccountId::from_str("user1").unwrap()
    }

    fn user2() -> AccountId {
        AccountId::from_str("user2").unwrap()
    }

    fn signer() -> AccountId {
        AccountId::from_str("signer").unwrap()
    }

    fn setup() -> (Contract, VMContextBuilder) {
        let mut context = VMContextBuilder::new();
        let contract = Contract::new(signer());

        context.current_account_id(current());
        context.account_balance(NearToken::from_near(1));
        context.attached_deposit(NearToken::from_yoctonear(0));
        context.predecessor_account_id(user1());
        context.block_timestamp(0);
        context.prepaid_gas(Gas::from_tgas(300));

        testing_env!(context.build());

        (contract, context)
    }

    fn input_request() -> InputRequest {
        InputRequest {
            allowed_executors: vec![Executor::Account {
                account_id: user1(),
            }],
            derivation_seed_number: 0,
            base_eip1559_payload: BaseEip1559TransactionPayload {
                to: "0x0000000000000000000000000000000000000000".to_string(),
                data: None,
                value: None,
                nonce: 0,
            },
        }
    }

    fn other_payload() -> OtherEip1559TransactionPayload {
        OtherEip1559TransactionPayload {
            chain_id: 1,
            gas: Some(U128(42_000)),
            max_fee_per_gas: U128(120_000),
            max_priority_fee_per_gas: U128(120_000),
        }
    }

    #[test]
    fn test_setup_succeeds() {
        setup();
    }

    #[test]
    fn test_set_signer_account_id() {
        let (mut contract, mut context) = setup();

        assert_eq!(contract.get_signer_account_id(), signer());

        context.predecessor_account_id(current());
        testing_env!(context.build());

        contract.set_signer_account_id(user2());

        assert_eq!(contract.get_signer_account_id(), user2());
    }

    #[should_panic = "ERR_FORBIDDEN_ONLY_OWNER"]
    #[test]
    fn test_set_signer_account_id_panics_on_wrong_predecessor() {
        let (mut contract, _) = setup();

        contract.set_signer_account_id(user2());
    }

    #[test]
    fn test_register_signature_request() {
        let (mut contract, _) = setup();

        let input_request = input_request();
        let request_id_1 = contract.register_signature_request(input_request.clone());
        let request_id_2 = contract.register_signature_request(input_request.clone());

        assert_ne!(request_id_1, request_id_2);
    }

    #[should_panic]
    #[test]
    fn test_register_signature_request_panics_on_empty_executors() {
        let (mut contract, _) = setup();

        let mut input_request = input_request();
        input_request.allowed_executors = vec![];

        contract.register_signature_request(input_request.clone());
    }

    #[should_panic]
    #[test]
    fn test_register_signature_request_panics_on_too_many_executors() {
        let (mut contract, _) = setup();

        let mut input_request = input_request();
        input_request.allowed_executors = vec![
            Executor::Account {
                account_id: user1()
            };
            32
        ];

        contract.register_signature_request(input_request.clone());
    }

    #[test]
    fn test_get_signature() {
        let (mut contract, _) = setup();

        let input_request = input_request();
        let request_id = contract.register_signature_request(input_request.clone());

        let other_payload = other_payload();
        contract.get_signature(request_id, other_payload);
    }

    #[should_panic = "ERR_NOT_FOUND"]
    #[test]
    fn test_get_signature_panics_on_unexisted_request() {
        let (mut contract, _) = setup();

        let other_payload = other_payload();
        contract.get_signature(100, other_payload);
    }

    #[should_panic = "ERR_FORBIDDEN"]
    #[test]
    fn test_get_signature_panics_on_non_allowed_executor() {
        let (mut contract, mut context) = setup();

        let input_request = input_request();
        let request_id = contract.register_signature_request(input_request.clone());

        context.predecessor_account_id(user2());
        testing_env!(context.build());

        let other_payload = other_payload();
        contract.get_signature(request_id, other_payload);
    }

    #[should_panic = "ERR_TIME_IS_UP"]
    #[test]
    fn test_get_signature_panics_on_deadline() {
        let (mut contract, mut context) = setup();

        let input_request = input_request();
        let request_id = contract.register_signature_request(input_request.clone());

        context.block_timestamp(15 * ONE_MINUTE_NANOS + 1);
        testing_env!(context.build());

        let other_payload = other_payload();
        contract.get_signature(request_id, other_payload);
    }

    #[should_panic = "ERR_INSUFFICIENT_GAS"]
    #[test]
    fn test_get_signature_panics_on_insufficient_gas() {
        let (mut contract, mut context) = setup();

        let input_request = input_request();
        let request_id = contract.register_signature_request(input_request.clone());

        // 260TGas - 1Gas
        context.prepaid_gas(Gas::from_tgas(260).checked_sub(Gas::from_gas(1)).unwrap());
        testing_env!(context.build());

        let other_payload = other_payload();
        contract.get_signature(request_id, other_payload);
    }
}
