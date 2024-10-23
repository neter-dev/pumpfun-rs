use solana_sdk::pubkey::Pubkey;

use solana_program::instruction::{AccountMeta, Instruction};
use crate::constants::{ASSOC_TOKEN_ACC_PROGRAM_ID, EVENT_AUTHORITY, PUMPFUN_FEE_RECIPIENT, PUMPFUN_GLOBAL, PUMPFUN_PROGRAM_ID, SYSTEM_RENT_PROGRAM_ID, TOKEN_PROGRAM_ID};


pub fn buy_amount_out_ix(
    mint: &Pubkey,
    bonding_curve: &Pubkey,
    associated_bonding_curve: &Pubkey,
    wallet: &Pubkey,
    associated_token_account: &Pubkey,
    amount_out: u64,
    max_amount_in_sol: u64) -> Instruction {

    let accounts = vec![
        AccountMeta::new_readonly(PUMPFUN_GLOBAL, false),
        AccountMeta::new(PUMPFUN_FEE_RECIPIENT, false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(*bonding_curve, false),
        AccountMeta::new(*associated_bonding_curve, false),
        AccountMeta::new(*associated_token_account, false),
        AccountMeta::new(*wallet, true),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_RENT_PROGRAM_ID, false),
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),
        AccountMeta::new_readonly(PUMPFUN_PROGRAM_ID, false)
    ];

    let mut data: Vec<u8> = Vec::new();
    data.extend_from_slice(&[0x66, 0x06, 0x3d, 0x12, 0x01, 0xda, 0xeb, 0xea]);
    data.extend_from_slice(&amount_out.to_le_bytes());
    data.extend_from_slice(&max_amount_in_sol.to_le_bytes());

    Instruction {
        program_id: PUMPFUN_PROGRAM_ID,
        accounts,
        data
    }
}


pub fn sell_amount_in_ix(
    mint: &Pubkey,
    bonding_curve: &Pubkey,
    associated_bonding_curve: &Pubkey,
    wallet: &Pubkey,
    associated_token_account: &Pubkey,
    amount_in: u64,
    min_amount_out_sol: u64) -> Instruction {

    let accounts = vec![
        AccountMeta::new_readonly(PUMPFUN_GLOBAL, false),
        AccountMeta::new(PUMPFUN_FEE_RECIPIENT, false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(*bonding_curve, false),
        AccountMeta::new(*associated_bonding_curve, false),
        AccountMeta::new(*associated_token_account, false),
        AccountMeta::new(*wallet, true),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(ASSOC_TOKEN_ACC_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),
        AccountMeta::new_readonly(PUMPFUN_PROGRAM_ID, false)
    ];

    let mut data: Vec<u8> = Vec::new();
    // 33e685a4017f83ad
    data.extend_from_slice(&[0x33, 0xe6, 0x85, 0xa4, 0x01, 0x7f, 0x83, 0xad]);
    data.extend_from_slice(&amount_in.to_le_bytes());
    data.extend_from_slice(&min_amount_out_sol.to_le_bytes());

    Instruction {
        program_id: PUMPFUN_PROGRAM_ID,
        accounts,
        data
    }
}
