use near_sdk::serde_json::{json, Value};
use near_workspaces::types::{AccountId, NearToken};
use near_workspaces::{Account, Contract};

pub const SEPOLIA_CHAIN_ID: u64 = 11155111;
pub const ARBITRUM_CHAIN_ID: u64 = 42161;
pub const ETHEREUM_CHAIN_ID: u64 = 1;

const MPC_CONTRACT_WASM_FILE_PATH: &str = "tests/res/mpc_contract.wasm";

fn get_mpc_request_scalars_for_chain(chain_id: u64) -> (&'static str, &'static str) {
    // epsilon persists the same across chains
    let epsilon = "10B4A3C8689179B34BE82E52D5C57434548A0A8514CDA91BAB9F4778A311286D";

    let payload_hash = match chain_id {
        SEPOLIA_CHAIN_ID => "562D144722DEBA4DA7630E9C494FFC8ACDC3347AAD329A61F6B7A824D7352BD0",
        ARBITRUM_CHAIN_ID => "813223E0E83162210A5C2EA3EF0ABC3651B3DDA211EF46F6588FD5E027628299",
        ETHEREUM_CHAIN_ID => "64E056DFA9BD5FD17E9DA1809EFF94CD4C6E6E9923CA4A1BC970827C0968C387",
        _ => panic!("The chain isn't supported!"),
    };

    (epsilon, payload_hash)
}

pub async fn deploy_abstract_dao_contract(root: &Account, mpc_contract_id: &AccountId) -> Contract {
    let wasm = near_workspaces::compile_project(".")
        .await
        .expect("Failed to compile contract WASM!");

    let account = root
        .create_subaccount("contract")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await
        .unwrap()
        .unwrap();

    let contract = account.deploy(&wasm).await.unwrap().unwrap();

    let init_result = root
        .call(contract.id(), "new")
        .args_json(json!({
            "mpc_contract_id": mpc_contract_id.to_string()
        }))
        .transact()
        .await
        .unwrap();

    dbg!(&init_result);

    assert!(
        init_result.is_success(),
        "Failed to initialize Abstract DAO contract!"
    );

    contract
}

pub async fn deploy_mpc_contract(root: &Account) -> Contract {
    // MPC contract is slightly modified!
    // Removed signature check inside fn respond() to be able to respond with a mock
    // as we don't care about signature validity during those tests
    let wasm = std::fs::read(MPC_CONTRACT_WASM_FILE_PATH).expect(
        format!(
            "Couldn't find Wasm file intended for Mpc contract at {MPC_CONTRACT_WASM_FILE_PATH}"
        )
        .as_str(),
    );

    let account = root
        .create_subaccount("mpc")
        .initial_balance(NearToken::from_near(20))
        .transact()
        .await
        .unwrap()
        .unwrap();

    let contract = account.deploy(&wasm).await.unwrap().unwrap();

    let init_result = contract
        .call("init_running")
        .args_json(json!({
            "epoch": 0,
            "threshold": 2,
            "participants": {
                "next_id": 3,
                "participants": {
                    "1.near": {
                        "account_id": "1.near",
                        "url": "127.0.0.1",
                        "cipher_pk": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        "sign_pk": "ed25519:2Y9Rz7ri9Js4jC3UagR226fNDaFDLRYrR3AX2edBR41r"
                    },
                    "2.near": {
                        "account_id": "2.near",
                        "url": "127.0.0.1",
                        "cipher_pk": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        "sign_pk": "ed25519:2Y9Rz7ri9Js4jC3UagR226fNDaFDLRYrR3AX2edBR41r"
                    },
                    "3.near": {
                        "account_id": "3.near",
                        "url": "127.0.0.1",
                        "cipher_pk": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        "sign_pk": "ed25519:2Y9Rz7ri9Js4jC3UagR226fNDaFDLRYrR3AX2edBR41r"
                    },
                },
                "account_to_participant_id": {
                    "1.near": 0,
                    "2.near": 1,
                    "3.near": 2
                }
            },
            "public_key": "secp256k1:54hU5wcCmVUPFWLDALXMh1fFToZsVXrx9BbTbHzSfQq1Kd1rJZi52iPa4QQxo6s5TgjWqgpY8HamYuUDzG6fAaUq"
        }))
        .transact()
        .await
        .unwrap();

    dbg!(&init_result);

    assert!(
        init_result.is_success(),
        "Failed to initialize MPC Contract!"
    );

    contract
}

pub async fn create_signature_request(user: &Account, contract_id: &AccountId) -> u64 {
    let register_request_result = user
        .call(contract_id, "register_signature_request")
        .deposit(NearToken::from_millinear(50)) // 0.05 NEAR
        .args_json(json!({
            "request": {
                "allowed_account_id": user.id().to_string(),
                "derivation_seed_number": 0,
                "transaction_payload": {
                    "to": "0xe2a01146FFfC8432497ae49A7a6cBa5B9Abd71A3",
                    "nonce": "0",
                    "function_data": {
                        "function_abi": {
                            "inputs": [
                                {
                                    "internalType": "uint256",
                                    "name": "_num",
                                    "type": "uint256"
                                }
                            ],
                            "name": "set",
                            "outputs": [],
                            "stateMutability": "nonpayable",
                            "type": "function"
                        },
                        "arguments": [
                            {
                                "Uint": "A97"
                            }
                        ]
                    }
                },
                "key_version": 0
            }
        }))
        .transact()
        .await
        .unwrap();

    println!("Register Request Result: {:?}", register_request_result);

    assert!(
        register_request_result.is_success(),
        "Function call register_signature_request wasn't successful!"
    );

    register_request_result.json::<Value>().unwrap()["request_id"]
        .as_u64()
        .unwrap()
}

pub async fn get_signature_and_validate(
    user: &Account,
    contract_id: &AccountId,
    request_id: u64,
    chain_id: u64,
    mpc_contract: &Contract,
) {
    let (epsilon, payload_hash) = get_mpc_request_scalars_for_chain(chain_id);

    let get_signature_tx = user
        .call(contract_id, "get_signature")
        .args_json(json!({
            "request_id": request_id,
            "other_payload": {
                "chain_id": chain_id,
                "max_fee_per_gas": "111551114121",
                "max_priority_fee_per_gas": "294111551111"
            }
        }))
        .deposit(NearToken::from_millinear(50)) // 0.05 NEAR
        .max_gas()
        .transact_async()
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let respond_result = mpc_contract
        .call("respond")
        .args_json(json!({
          "request": {
            "epsilon": {
              "scalar": epsilon
            },
            "payload_hash": {
              "scalar": payload_hash
            }
          },
          // The response object is just a mock
          "response": {
            "big_r": {
              "affine_point": "03214BB5B327CEC619FB0447C84E23E5DF462FD758D46F0A21A36EF9BC083EF53B"
            },
            "recovery_id": 0,
            "s": {
              "scalar": "314BA3D6CC3B41C255C857C1216FFCC9AE71A17C0B38146613D4C6EFE5416FC7"
            }
          }
        }))
        .max_gas()
        .transact()
        .await
        .unwrap();

    dbg!(&respond_result);

    assert!(
        respond_result.is_success(),
        "MPC Respond wasn't successful!"
    );

    let get_signature_result = get_signature_tx.await.unwrap();
    dbg!(&get_signature_result);

    assert!(
        get_signature_result.is_success(),
        "Get signature wasn't successful!"
    );

    let response = get_signature_result.json::<Value>().unwrap();

    assert!(response["tx"].is_string());
    assert_eq!(
        response["signature"]["big_r"]["affine_point"],
        "03214BB5B327CEC619FB0447C84E23E5DF462FD758D46F0A21A36EF9BC083EF53B"
    );
    assert_eq!(response["signature"]["recovery_id"], 0);
    assert_eq!(
        response["signature"]["s"]["scalar"],
        "314BA3D6CC3B41C255C857C1216FFCC9AE71A17C0B38146613D4C6EFE5416FC7"
    );
}
