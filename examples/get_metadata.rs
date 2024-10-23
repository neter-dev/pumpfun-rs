use std::env;
use std::str::FromStr;

use solana_program::pubkey::Pubkey;

use pumpfun_rs::metadata::get_token_metadata;

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: ./get_metadata <TOKEN_MINT>");
        return;
    }
    let token: Pubkey = Pubkey::from_str(&args[1]).unwrap();

    let metadata = get_token_metadata(&token).await;
    println!("Token MetaData: {:#?}", metadata);
}
