use std::collections::HashMap;

use solana_sdk::pubkey::Pubkey;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::hash::Hash;
use solana_program::instruction::Instruction;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::{get_associated_token_address_with_program_id, instruction};
use spl_token::instruction::close_account;

use crate::curve::{derive_bonding_curve_accounts, get_bonding_curve_state};
use crate::instructions::{buy_amount_out_ix, sell_amount_in_ix};

pub mod curve;
pub mod constants;
pub mod metadata;
pub mod instructions;

pub struct PumpFunClient {
    derived_account_cache: HashMap<Pubkey, (Pubkey, Pubkey)>,
    rpc_client: RpcClient,
    wallet: Keypair,
    wallet_pubkey: Pubkey,
}


impl PumpFunClient {

    pub fn new(rpc_client: RpcClient, wallet: &Keypair) -> PumpFunClient {

        PumpFunClient {
            derived_account_cache: HashMap::new(),
            wallet: wallet.insecure_clone(),
            wallet_pubkey: wallet.pubkey(),
            rpc_client,
        }
    }

    fn get_derived_accounts(&mut self, mint: &Pubkey) -> (Pubkey, Pubkey) {
        if let Some(accounts) = self.derived_account_cache.get(mint) {
            return *accounts;
        }

        let accounts = derive_bonding_curve_accounts(mint);
        self.derived_account_cache.insert(*mint, accounts);
        accounts
    }

    pub async fn get_price(&mut self, mint: &Pubkey) -> Result<f32, Box<dyn std::error::Error>> {
        let (bonding_curve, _) = self.get_derived_accounts(mint);
        let state = get_bonding_curve_state(&self.rpc_client, &bonding_curve).await?;

        if state.complete {
            return Err("Curve is complete. Check Raydium reserves for price.".into());
        }

        Ok(state.price())
    }
    
    pub async fn get_balance(&self, mint: &Pubkey) -> Result<u64, Box<dyn std::error::Error>> {
        // Helper function to get the balance of a token account
        
        let token_ata = get_associated_token_address_with_program_id(&self.wallet_pubkey, mint, &spl_token::id());

        match self.rpc_client.get_token_account_balance_with_commitment(&token_ata, CommitmentConfig::confirmed()).await {
            Ok(balance) => {
                Ok(balance.value.amount.as_str().parse::<u64>().unwrap())
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }

    pub async fn buy(&mut self, mint: &Pubkey, amount_in: u64, slippage: f32, create_token_ata: bool, priority_fee: u64, blockhash: &Hash) -> Result<Signature, Box<dyn std::error::Error>> {

        match self.create_buy_transaction(mint, amount_in, slippage, create_token_ata, priority_fee, blockhash).await {
            Ok(tx) => {
                match self.rpc_client.send_transaction_with_config(
                    &tx,
                    RpcSendTransactionConfig {
                        skip_preflight: false,
                        preflight_commitment: Some(CommitmentLevel::Confirmed),
                        .. RpcSendTransactionConfig::default()
                    }
                ).await {
                    Ok(signature) => {
                        Ok(signature)
                    },
                    Err(e) => Err(e.into()),
                }
            },
            Err(e) => Err(e),
        }
    }

    pub async fn create_buy_transaction(&mut self, mint: &Pubkey, amount_in_sol: u64, slippage: f32, create_token_ata: bool, priority_fee: u64, blockhash: &Hash) -> Result<Transaction, Box<dyn std::error::Error>> {
        let (bonding_curve, associated_bonding_curve) = self.get_derived_accounts(mint);
        let state = get_bonding_curve_state(&self.rpc_client, &bonding_curve).await?;

        if state.complete {
            return Err("Curve is complete. Cannot buy on Pumpfun.".into());
        }

        let price = state.price();

        let amount_out = (amount_in_sol as f32 / price / 1_000.0) as u64;
        let max_amount_in_sol = (amount_in_sol as f32 * (1.0 + slippage)) as u64;

        let mut ixs: Vec<Instruction> = Vec::new();

        if priority_fee > 0 {
            ixs.push(ComputeBudgetInstruction::set_compute_unit_price(priority_fee));
        }

        let token_ata = if create_token_ata {
            let create_ata_ix = instruction::create_associated_token_account(
                &self.wallet_pubkey,
                &self.wallet_pubkey,
                mint,
                &constants::TOKEN_PROGRAM_ID,
            );

            ixs.push(create_ata_ix.clone());
            create_ata_ix.accounts[1].pubkey
        } else {
            get_associated_token_address_with_program_id(&self.wallet_pubkey, mint, &spl_token::id())
        };

        ixs.push(buy_amount_out_ix(
            mint,
            &bonding_curve,
            &associated_bonding_curve,
            &self.wallet_pubkey,
            &token_ata,
            amount_out,
            max_amount_in_sol));

        Ok(Transaction::new_signed_with_payer(&ixs.to_vec(), Some(&self.wallet.pubkey()), &[&self.wallet], *blockhash))
    }
    
    pub async fn create_sell_transaction(&mut self, mint: &Pubkey, amount_in_token: u64, slippage: f32, close_token_ata: bool, priority_fee: u64, blockhash: &Hash) -> Result<Transaction, Box<dyn std::error::Error>> {
        let (bonding_curve, associated_bonding_curve) = self.get_derived_accounts(mint);
        let state = get_bonding_curve_state(&self.rpc_client, &bonding_curve).await?;

        if state.complete {
            return Err("Curve is complete. Cannot sell on Pumpfun.".into());
        }

        let price = state.price();
        
        let amount_out_sol = (amount_in_token as f32 * price * 1000.0) as u64;
        let min_amount_out_sol = amount_out_sol - (amount_out_sol as f32 * slippage) as u64;
        
        println!("Amount in token: {}", amount_in_token);
        println!("Amount out sol: {}", amount_out_sol);
        println!("Min amount out sol: {}", min_amount_out_sol);

        let mut ixs: Vec<Instruction> = Vec::new();

        if priority_fee > 0 {
            ixs.push(ComputeBudgetInstruction::set_compute_unit_price(priority_fee));
        }
        
        let token_ata = get_associated_token_address_with_program_id(&self.wallet_pubkey, mint, &spl_token::id());

        ixs.push(sell_amount_in_ix(
            mint,
            &bonding_curve,
            &associated_bonding_curve,
            &self.wallet_pubkey,
            &token_ata,
            amount_in_token,
            min_amount_out_sol));

        if close_token_ata {
            ixs.push(
                close_account(
                    &constants::TOKEN_PROGRAM_ID,
                    &token_ata,
                    &self.wallet_pubkey,
                    &self.wallet_pubkey,
                    &[], // no rent account
                )?
            );
        };

        Ok(Transaction::new_signed_with_payer(&ixs.to_vec(), Some(&self.wallet.pubkey()), &[&self.wallet], *blockhash))
    }

    pub async fn sell(&mut self, mint: &Pubkey, amount_in: u64, slippage: f32, close_token_ata: bool, priority_fee: u64, blockhash: &Hash) -> Result<Signature, Box<dyn std::error::Error>> {

        match self.create_sell_transaction(mint, amount_in, slippage, close_token_ata, priority_fee, blockhash).await {
            Ok(tx) => {
                match self.rpc_client.send_transaction_with_config(
                    &tx,
                    RpcSendTransactionConfig {
                        skip_preflight: false,
                        preflight_commitment: Some(CommitmentLevel::Confirmed),
                        .. RpcSendTransactionConfig::default()
                    }
                ).await {
                    Ok(signature) => {
                        Ok(signature)
                    },
                    Err(e) => Err(e.into()),
                }
            },
            Err(e) => Err(e),
        }
    }    
}
