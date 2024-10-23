# PumpFun-RS

PumpFun-RS is a simple Rust library for interacting with the PumpFun DeFi platform on the Solana blockchain.

## Disclaimer

This library is still in development and should be used with caution. 
By using this library you agree that the author is not responsible for any loss of funds or any other damages incurred.

Crypto trading is risky! Becareful out there.

## Features

- Retrieve token metadata.
- Derive bonding curve accounts
- Fetch curve state (incl. price)
- Build swap instructions
- Helpers to buy and sell tokens.
- Example code.

## Installation

```
cargo add pumpfun-rs
```

## Examples

Full examples are in the [examples](examples) folder.

```rust
use std::env;
use std::str::FromStr;

use solana_sdk::signature::Keypair;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_program::pubkey::Pubkey;
use solana_client::nonblocking::rpc_client::RpcClient;

use pumpfun_rs::PumpFunClient;

#[tokio::main]
async fn main() {
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let wallet = env::var("WALLET").expect("WALLET must be set");
    let wallet = Keypair::from_base58_string(&wallet);

    let rpc_client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let blockhash = rpc_client.get_latest_blockhash().await.expect("Failed to get blockhash");

    let token: Pubkey = Pubkey::from_str("A_TOKEN_ADDRESS_pump").unwrap();

    let amount_in = (0.001 * 1_000_000_000.0) as u64;       // 0.001 SOL

    let mut pumpfun = PumpFunClient::new(rpc_client, &wallet);

    match pumpfun.buy(&token, amount_in, 0.10, true, 1_000_000, &blockhash).await {
        Ok(result) => {
            println!("Buy Signature: {:#?}", result);
        },
        Err(e) => {
            println!("Error: {:#?}", e);
        }
    }
}

```

## License

This project is licensed under the MIT License.
```
