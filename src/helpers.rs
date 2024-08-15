use ethers_core::types::transaction::eip2930::AccessList;
use ethers_core::types::Eip1559TransactionRequest;
use ethers_core::utils::keccak256;
use near_sdk::serde_json::json;
use near_sdk::{env, require, AccountId, Gas, NearToken, Promise, StorageUsage};

use crate::constants::GAS_FOR_PROMISE;
use crate::primitives::{BaseEip1559TransactionPayload, OtherEip1559TransactionPayload, Request};

pub fn create_derivation_path(seed_number: u32) -> String {
    format!("{}-{}", env::predecessor_account_id(), seed_number)
}

pub fn refund_unused_deposit(used_deposit: NearToken) {
    let refund = env::attached_deposit().checked_sub(used_deposit).unwrap();

    if refund > NearToken::from_yoctonear(1) {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

pub fn assert_deposit(min_deposit: NearToken) {
    require!(
        env::attached_deposit() >= min_deposit,
        format!(
            "Deposited amount must be bigger than {} yocto",
            min_deposit.as_yoctonear()
        )
    );
}

pub fn assert_gas(min_gas: Gas) {
    require!(env::prepaid_gas() >= min_gas, "ERR_INSUFFICIENT_GAS");
}

pub fn calculate_deposit_for_used_storage(used_storage: StorageUsage) -> NearToken {
    env::storage_byte_cost()
        .checked_mul(used_storage as u128)
        .expect("ERR_STORAGE_DEPOSIT_CALC")
}

fn create_eip1559_tx(
    base_payload: BaseEip1559TransactionPayload,
    other_payload: OtherEip1559TransactionPayload,
) -> Eip1559TransactionRequest {
    let base_tx: Eip1559TransactionRequest = base_payload.into();
    let other_tx: Eip1559TransactionRequest = other_payload.into();

    Eip1559TransactionRequest {
        // Base Tx
        to: base_tx.to,
        data: base_tx.data,
        value: base_tx.value,
        nonce: base_tx.nonce,
        // Other Tx provided by requestor
        chain_id: other_tx.chain_id,
        gas: other_tx.gas,
        max_fee_per_gas: other_tx.max_fee_per_gas,
        max_priority_fee_per_gas: other_tx.max_priority_fee_per_gas,
        // Unused
        from: None,
        access_list: AccessList::default(),
    }
}

fn build_tx_payload(tx: Eip1559TransactionRequest) -> [u8; 32] {
    // byte "2" stands for EIP-1559 Type
    let mut vec = vec![2u8];
    vec.extend(tx.rlp().to_vec());

    keccak256(vec)
}

pub fn create_tx_and_args_for_sign(
    request: Request,
    other_payload: OtherEip1559TransactionPayload,
) -> (Eip1559TransactionRequest, Vec<u8>) {
    let tx = create_eip1559_tx(request.payload.clone(), other_payload);
    let payload = build_tx_payload(tx.clone());

    let args = json!({
        "request": {
            "payload": payload,
            "path": request.derivation_path,
            "key_version": request.key_version
        }
    })
    .to_string()
    .into_bytes();

    (tx, args)
}

pub fn create_sign_promise(account_id: AccountId, args: Vec<u8>) -> Promise {
    let function = "sign".to_owned();
    let deposit = env::attached_deposit();
    // calculate unused gas
    let gas = env::prepaid_gas()
        .checked_sub(env::used_gas())
        .unwrap()
        // some Gas will be used to create Promise itself
        .checked_sub(GAS_FOR_PROMISE)
        .unwrap();

    Promise::new(account_id).function_call(function, args, deposit, gas)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethers_core::types::{NameOrAddress, H160, U256, U64};
    use near_sdk::{
        json_types::U128, test_utils::VMContextBuilder, testing_env, AccountId, NearToken,
    };

    use super::*;

    #[test]
    fn test_derivation_path_creation() {
        let mut context = VMContextBuilder::new();
        context.predecessor_account_id(AccountId::from_str("account").unwrap());

        testing_env!(context.build());

        let derivation_path = create_derivation_path(11111111);

        assert_eq!(derivation_path, "account-11111111");
    }

    #[test]
    fn test_assert_enough_deposit() {
        let mut context = VMContextBuilder::new();
        context.attached_deposit(NearToken::from_yoctonear(100));

        testing_env!(context.build());

        assert_deposit(NearToken::from_yoctonear(100));
    }

    #[should_panic]
    #[test]
    fn test_assert_small_deposit() {
        let mut context = VMContextBuilder::new();
        context.attached_deposit(NearToken::from_yoctonear(100));

        testing_env!(context.build());

        assert_deposit(NearToken::from_yoctonear(200));
    }

    #[test]
    fn test_assert_enough_gas() {
        let mut context = VMContextBuilder::new();
        context.prepaid_gas(Gas::from_tgas(30));

        testing_env!(context.build());

        assert_gas(Gas::from_tgas(30));
    }

    #[should_panic]
    #[test]
    fn test_assert_small_gas() {
        let mut context = VMContextBuilder::new();
        context.prepaid_gas(Gas::from_tgas(30));

        testing_env!(context.build());

        assert_gas(Gas::from_tgas(60));
    }

    #[test]
    fn test_calculate_deposit_for_used_storage() {
        let mut context = VMContextBuilder::new();
        context.prepaid_gas(Gas::from_tgas(30));

        testing_env!(context.build());

        let deposit = calculate_deposit_for_used_storage(100_000);

        assert_eq!(deposit, NearToken::from_near(1));
    }

    #[test]
    fn test_create_eip1559_tx() {
        let base_payload = BaseEip1559TransactionPayload {
            to: "0x0000000000000000000000000000000000000000".to_string(),
            nonce: U128(2 * u64::MAX as u128 + 5),
            value: Some(U128(u64::MAX as u128 - 125)),
            data: Some("0x2386f26fc10000".to_string()),
        };

        let other_payload = OtherEip1559TransactionPayload {
            chain_id: 1111,
            gas: Some(U128(42_000)),
            max_fee_per_gas: U128(120_000),
            max_priority_fee_per_gas: U128(120_000),
        };

        let tx = create_eip1559_tx(base_payload, other_payload);

        // base fields
        assert_eq!(
            tx.to,
            Some(NameOrAddress::Address(H160::from_slice(&[0u8; 20])))
        );
        // (2 * (u64::MAX + 1)) + 3
        assert_eq!(tx.nonce, Some(U256([3, 2, 0, 0])));
        assert_eq!(tx.value, Some(U256([u64::MAX - 125, 0, 0, 0])));
        assert!(tx.data.is_some());

        // other fields
        assert_eq!(tx.chain_id, Some(U64([1111])));
        assert_eq!(tx.gas, Some(U256([42_000, 0, 0, 0])));
        assert_eq!(tx.max_fee_per_gas, Some(U256([120_000, 0, 0, 0])));
        assert_eq!(tx.max_priority_fee_per_gas, Some(U256([120_000, 0, 0, 0])));
    }

    #[test]
    fn test_build_tx_payload() {
        let base_payload = BaseEip1559TransactionPayload {
            to: "0x427F9620Be0fe8Db2d840E2b6145D1CF2975bcaD".to_string(),
            value: Some(U128(1_000_000_000_000_000)),
            data: None,
            nonce: U128(0),
        };

        let other_payload = OtherEip1559TransactionPayload {
            gas: Some(U128(21_000)),
            max_fee_per_gas: U128(21_814_571_193),
            max_priority_fee_per_gas: U128(669_340_333),
            chain_id: 11_155_111,
        };

        let tx = create_eip1559_tx(base_payload, other_payload);

        let payload = build_tx_payload(tx);

        assert_eq!(
            payload,
            [
                196, 219, 238, 31, 254, 194, 212, 22, 3, 0, 13, 6, 13, 25, 120, 218, 26, 251, 37,
                243, 151, 233, 169, 4, 235, 115, 236, 84, 195, 81, 213, 124
            ]
        );
    }
}
