# Soroban Escrow Contract

## Overview

Soroban smart contract that allows users to lock up tokens for a specified period, after which they can unlock and retrieve their tokens. The contract ensures that the tokens remain inaccessible until the designated time.

## Getting Started

### Prerequisites

- **Rust & Soroban Environment**: Set up the environment for building, deploying and interacting with Soroban contracts. Detailed instructions are available in the [Stellar Developer Documentation](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup).

- **Stellar Asset Contract (SAC)**: Deploy the SAC for the Stellar asset intended to be used in the contract using the Stellar CLI to enable its use in the escrow contract; refer to the [Deploy the Stellar Asset Contract for a Stellar asset](https://developers.stellar.org/docs/build/guides/cli/deploy-stellar-asset-contract) guide for instructions.

### Usage

- **Build**: Compile the contract to a WASM file.

  ```
  stellar contract build
  ```

- `__constructor`: Initialize the contract with a token address and maximum lockup duration, store the token and duration in instance storage, create an empty escrow list in persistent storage, and prevent reinitialization by checking if the token is already set.

  ```
  stellar contract deploy --wasm target/wasm32v1-none/release/soroban_escrow_contract.wasm --source [DEPLOYER_SECRET_KEY] --network testnet -- --token [STELLAR_ASSET_CONTRACT] --max_lockup_duration [TIME_IN_SECONDS]
  ```

- `lock`: Lock a specified amount of tokens until a given timestamp, require account authentication, confirm that the claim timestamp is future-dated and within the maximum lockup duration, transfer tokens to the contract, record escrow details, and emit a lock event.

  ```
  stellar contract invoke --id [ESCROW_CONTRACT_ADDRESS] --source [CALLER_PRIVATE_KEY] --network testnet -- lock --account [CALLER_PUBLIC_KEY] --amount [i128_INTEGER] --claim_after [TIME_IN_SECONDS]
  ```

- `unlock` : Unlock and return tokens to the user after the claim timestamp, require account authentication, check that the escrow exists and the current time exceeds the claim timestamp, transfer tokens back, delete escrow data, and emit an unlock event.

  ```
  stellar contract invoke --id [ESCROW_CONTRACT_ADDRESS] --source [CALLER_PRIVATE_KEY] --network testnet -- unlock --account [CALLER_PUBLIC_KEY]
  ```

- `extend_ttl`: Extend the contract’s instance storage time-to-live (TTL) to ensure data persistence over time.

  ```
  stellar contract invoke --id [ESCROW_CONTRACT_ADDRESS] --source [CALLER_PRIVATE_KEY] --network testnet -- extend_ttl --ttl [LEDGERS_TO_EXTEND]
  ```

- `get_escrow`: Retrieve escrow details for a specific account, return None if no escrow exists, and indicate whether it is possible to unlock now.

  ```
  stellar contract invoke --id [ESCROW_CONTRACT_ADDRESS] --source [CALLER_PRIVATE_KEY] --network testnet -- get_escrow --account [ACCOUNT_PUBLIC_KEY]
  ```

- `get_escrows`: List details of all active escrows, and indicate for each whether it is possible to unlock at the current time.

  ```
  stellar contract invoke --id [ESCROW_CONTRACT_ADDRESS] --source [CALLER_PRIVATE_KEY] --network testnet -- get_escrows
  ```

## License

This project is licensed under the [MIT License](LICENSE).
