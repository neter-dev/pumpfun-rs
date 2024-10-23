use std::env;
use std::str::FromStr;
use dotenvy::dotenv;


use solana_sdk::signature::Keypair;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_program::pubkey::Pubkey;
use solana_client::nonblocking::rpc_client::RpcClient;

use pumpfun_rs::PumpFunClient;

const DEFAULT_SLIPPAGE: f32 = 0.10;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: ./buy_token <TOKEN_MINT> <AMOUNT_IN_SOL> [SLIPPAGE]");
        println!("  TOKEN_MINT: The token you want to buy");
        println!("  AMOUNT_IN_SOL: SOL amount as a float, e.g. 1.0 = 1 SOL");
        println!("  SLIPPAGE: Optional slippage tolerance as a float, default is 0.10 (10%)");
        return;
    }

    dotenv().expect("Failed to load .env file");
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");

    let wallet = env::var("WALLET").expect("WALLET must be set");
    let wallet = Keypair::from_base58_string(&wallet);

    let token: Pubkey = Pubkey::from_str(&args[1]).unwrap();
    let amount_in = args[2].parse::<f32>().expect("Error parsing amount");
    let amount_in = (amount_in * 1_000_000_000.0) as u64;
    let slippage = if args.len() > 3 { args[3].parse::<f32>().unwrap() } else { DEFAULT_SLIPPAGE };

    let rpc_client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let blockhash = rpc_client.get_latest_blockhash().await.expect("Failed to get blockhash");

    let mut pumpfun = PumpFunClient::new(rpc_client, &wallet);

    let result = pumpfun.buy(&token, amount_in, slippage, true, 1_000_000, &blockhash).await;

    println!("Buy Result: {:#?}", result);
}
