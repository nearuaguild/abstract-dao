# Near Multi-Chain DAO Governance Contract

The smart contract is designed to act as an intermediary between Decentralized Organizations (such as DAOs or Multisig contracts) and a Multi-Party Computation (MPC) contract. Its primary purpose is to streamline the governance process for DAO councils by allowing them to vote on proposals once and automatically generate the necessary signatures for the same payload across multiple Ethereum Virtual Machine (EVM) compatible chains—differing only by the chain ID.

## Environments

Currently, there're 2 environments:

1. Testnet: `abstract-dao.testnet`
2. Dev (unstable): `dev.abstract-dao.testnet`

## How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build --no-abi --no-docker
```

## How to Test Locally?

The following command runs both unit and integration tests:

```bash
cargo test
```

## How to Deploy?

Deployment is automated with GitHub Actions CI/CD pipeline.
To deploy manually, install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near deploy --no-abi <account-id>
```

## How To Use?

1. A proposal is created and submitted to the organization (DAO or multisig) for review.

2. Members cast their votes on the proposal (either approve, or reject)

3. Once approved, the institution turns to the Governance Contract to record their intention to generate a signature for a specific payload and grant permission to some account to execute it

4. For each target EVM-compatible chain, the eligible account interacts with the Governance Contract to construct a signature specific to that chain

## API

To execute a command from the example, install [`near-cli`](https://near.cli.rs)

### `register_signature_request()`

This is one of the main functions of the contract. It records user's intention to generate a signature for a specific payload and grants permission to some account to execute it later

```rs
pub fn register_signature_request(&mut self, request: InputRequest) -> RequestId
```

#### Request Example

```bash
near contract call-function as-transaction abstract-dao.testnet register_signature_request json-args '{
    "request": {
        "allowed_account_id": "<eligible-account-id>",
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
        }
    }
}' prepaid-gas '100.0 Tgas' attached-deposit '0.1 NEAR' sign-as <dao-account-id> network-config testnet
```

- `<eligible-account-id>` is the user who will be allowed to get signature later
- `<dao-account-id>` is the institution's account for which signature is generated
- Integer arguments must be base64 encoded

#### Response Example

```json
{
  "deadline": 1728984746246497739,
  "derivation_path": "denbite.testnet-0",
  "mpc_account_id": "v1.signer-prod.testnet",
  "request_id": 0
}
```

- `deadline` is Unix timestamp in nanoseconds

### `get_signature()`

This is one of the main functions of the contract. It validates predecessor's permissions, converts payload into EIP-1559 transaction, and transmits further to MPC Contract where the signature is created

```rs
pub fn get_signature(&mut self, request_id: RequestId, other_payload: OtherEip1559TransactionPayload) -> Promise
```

#### Request Example

```bash
near contract call-function as-transaction abstract-dao.testnet get_signature json-args '{
    "request_id": <request-id>,
    "other_payload": {
        "chain_id": 11155111,
        "max_fee_per_gas": "111551114121",
        "max_priority_fee_per_gas": "294111551111"
    }
}' prepaid-gas '300.0 Tgas' attached-deposit '0.05 NEAR' sign-as <eligible-account-id> network-config testnet
```

- `<request_id>` is returned in response of `register_signature_request()`
- `<eligible-account-id>` must have permission to run `get_signature()`, otherwise it will throw forbidden error
- Prepaid gas must be bigger than 250TGas

#### Response Example

```json
{
  "signature": {
    "big_r": {
      "affine_point": "02D532992B0ECBF67800DB14E04530D9BA55609AD31213CC7ABDB554E8FDA986D3"
    },
    "recovery_id": 1,
    "s": {
      "scalar": "40E81711B8174712B9F34B2540EE0F642802387D15543CBFC84211BB04B83AC3"
    }
  },
  "tx": "0x02f85083aa36a702850485034c878517a4eb0789829dd094e2a01146fffc8432497ae49a7a6cba5b9abd71a380a460fe47b1000000000000000000000000000000000000000000000000000000000000a84bc0"
}
```

- `tx` is hex-encoded payload of EIP-1559 transaction
- `signature` is derived by [MPC Contract](https://github.com/near/mpc/tree/develop/chain-signatures/contract) (see this [repository](https://github.com/nearuaguild/multichain-dao-scripts) to understand how it can be easily relayed to the EVM chain)

## Useful Links

- [multichain-dao-scripts](https://github.com/nearuaguild/multichain-dao-scripts) - The script to relay signed EIP-1559 transaction directly to EVM chain
- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Iteract with NEAR blockchain from command line
- [NEAR Rust SDK Documentation](https://docs.near.org/sdk/rust/introduction)
- [NEAR Documentation](https://docs.near.org)
- [NEAR StackOverflow](https://stackoverflow.com/questions/tagged/nearprotocol)
- [NEAR Discord](https://near.chat)
- [NEAR Telegram Developers Community Group](https://t.me/neardev)
- NEAR DevHub: [Telegram](https://t.me/neardevhub), [Twitter](https://twitter.com/neardevhub)
