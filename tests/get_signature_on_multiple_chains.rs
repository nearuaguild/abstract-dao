pub mod common;
use common::{
    create_signature_request, deploy_abstract_dao_contract, deploy_mpc_contract,
    get_signature_and_validate, ARBITRUM_CHAIN_ID, ETHEREUM_CHAIN_ID, SEPOLIA_CHAIN_ID,
};
use near_workspaces::types::NearToken;

#[tokio::test]
async fn test_sign_eip1559_payload_on_many_chains() {
    let worker = near_workspaces::sandbox().await.unwrap();
    let root = worker.root_account().unwrap();

    let mpc_contract = deploy_mpc_contract(&root).await;

    let contract = deploy_abstract_dao_contract(&root, mpc_contract.id()).await;
    println!("Contract ID: {}", contract.id());

    let user = root
        .create_subaccount("user")
        .initial_balance(NearToken::from_near(5))
        .transact()
        .await
        .unwrap()
        .unwrap();
    println!("User ID: {}", user.id());

    let request_id = create_signature_request(&user, contract.id()).await;
    println!("Request ID: {:?}", request_id);

    // Generate signatures for many chains with the same payload
    let chains = [SEPOLIA_CHAIN_ID, ARBITRUM_CHAIN_ID, ETHEREUM_CHAIN_ID];
    for chain in chains {
        get_signature_and_validate(&user, contract.id(), request_id, chain, &mpc_contract).await;
    }
}
