use std::env;
use std::str::FromStr;
use dotenvy::dotenv;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;

use pumpfun_rs::curve::{derive_bonding_curve_accounts, get_bonding_curve_state};

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: ./get_curvestate <TOKEN_MINT>");
        return;
    }
    let token: Pubkey = Pubkey::from_str(&args[1]).unwrap();

    dotenv().expect("Failed to load .env file");
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let rpc_client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let (bonding_curve, _) = derive_bonding_curve_accounts(&token);
    let state = get_bonding_curve_state(&rpc_client, &bonding_curve).await;
    
    if state.is_err() {
        println!("Error: {:#?}", state.err().unwrap());
        return;
    }
    
    let state = state.unwrap();

    println!("Curve State: {:#?}", state);
    println!("Price: {:#?}", state.price());

}