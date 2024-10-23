use std::env;
use std::str::FromStr;
use dotenvy::dotenv;


use solana_sdk::signature::Keypair;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_program::pubkey::Pubkey;
use solana_client::nonblocking::rpc_client::RpcClient;
use pumpfun_rs::metadata::get_token_metadata;
use pumpfun_rs::PumpFunClient;

const DEFAULT_SLIPPAGE: f32 = 0.10;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: ./sell_token <TOKEN_MINT> <AMOUNT_TOKEN> [SLIPPAGE]");
        println!("  TOKEN_MINT: The mint of the token you want to buy");
        println!("  SLIPPAGE: Optional slippage tolerance as a float, default is 0.10 (10%)");
        return;
    }

    dotenv().expect("Failed to load .env file");
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");

    let wallet = env::var("WALLET").expect("WALLET must be set");
    let wallet = Keypair::from_base58_string(&wallet);

    let token: Pubkey = Pubkey::from_str(&args[1]).unwrap();
    let slippage = if args.len() > 3 { args[3].parse::<f32>().unwrap() } else { DEFAULT_SLIPPAGE };
    
    if !(0.0..=1.0).contains(&slippage) {
        println!("Invalid slippage");
        return;
    }

    let mut pumpfun = PumpFunClient::new(RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed()), &wallet);

    let balance = pumpfun.get_balance(&token).await.expect("Failed to get balance");
    let metadata = get_token_metadata(&token).await.expect("Failed to get metadata");

    println!("Token {:} [{:}] - Balance {:}", metadata.name, metadata.symbol, balance);
    println!("Enter amount to sell [Enter for all]: ");
    
    let mut line = String::new();
    
    let amount_in = match std::io::stdin().read_line(&mut line) {
        Ok(_) => {
            if line.trim().is_empty() {
                balance
            } else {
                line.trim().parse::<u64>().expect("Error parsing amount")
            }
        },
        Err(_) => {
            println!("Invalid amount to sell");
            return;
        }
    };
    
    if amount_in < 1 || amount_in > balance {
        println!("Invalid amount to sell");
        return;
    }

    let rpc_client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let blockhash = rpc_client.get_latest_blockhash().await.expect("Failed to get blockhash");
    
    let tx = pumpfun.sell(&token, amount_in, slippage, amount_in == balance, 1_000_000, &blockhash).await;
    println!("Sell Result: {:#?}", tx);
}
