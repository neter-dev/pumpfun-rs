use std::error::Error;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;

use crate::constants::PUMPFUN_PROGRAM_ID;

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct CurveState {
    _signature: [u8; 8],                // [0x17, 0xb7, 0xf8, 0x37, 0x60, 0xd8, 0xac, 0x60]
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
}

impl CurveState {
    pub fn price(&self) -> f32 {
        
        if self.complete || self.virtual_token_reserves == 0 || self.virtual_sol_reserves == 0 {
            return 0.0;
        }
        
        (self.virtual_sol_reserves as f32 / 10f32.powi(9)) / (self.virtual_token_reserves as f32 / 10f32.powi(6))
    }
}

pub fn derive_bonding_curve_accounts(mint: &Pubkey) -> (Pubkey, Pubkey) {

    let (bonding_curve, _) = Pubkey::find_program_address(
        &["bonding-curve".as_bytes(), mint.as_ref()],
        &PUMPFUN_PROGRAM_ID
    );

    let associated_bonding_curve = get_associated_token_address(&bonding_curve, mint);

    (bonding_curve, associated_bonding_curve)
}

pub async fn get_bonding_curve_state(client: &RpcClient, bonding_curve: &Pubkey) -> Result<CurveState, Box<dyn Error>> {

    match client.get_account_data(bonding_curve).await {
        Ok(account_data) => {
            let curve_state = CurveState::try_from_slice(&account_data)?;
            Ok(curve_state)
        },
        Err(e) => {
            Err(format!("Error getting account data: {:?}", e).into())
        }
    }
}
