use solana_client::client_error::reqwest;
use solana_client::client_error::reqwest::StatusCode;
use solana_sdk::pubkey::Pubkey;
use serde::{Deserialize, Serialize};

use log::error;
use std::error::Error;
use std::str::FromStr;

pub fn deserialize_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Pubkey::from_str(&s).map_err(serde::de::Error::custom)
}

pub fn deserialize_option_pubkey<'de, D>(deserializer: D) -> Result<Option<Pubkey>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = serde::Deserialize::deserialize(deserializer)?;
    match s {
        Some(s) => Pubkey::from_str(&s).map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TokenMetaData {

    #[serde(deserialize_with = "deserialize_pubkey")]
    pub mint: Pubkey,

    #[serde(deserialize_with = "deserialize_pubkey")]
    pub bonding_curve: Pubkey,

    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image_uri: String,
    pub video_uri: Option<String>,
    pub metadata_uri: String,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    
    #[serde(deserialize_with = "deserialize_pubkey")]
    pub associated_bonding_curve: Pubkey,
    
    #[serde(deserialize_with = "deserialize_pubkey")]
    pub creator: Pubkey,
    pub created_timestamp: u64,

    #[serde(deserialize_with = "deserialize_option_pubkey")]
    pub raydium_pool: Option<Pubkey>,
    pub complete: bool,
    pub virtual_sol_reserves: i64,
    pub virtual_token_reserves: i64,
    pub total_supply: i64,
    pub website: Option<String>,
    pub show_name: bool,
    pub king_of_the_hill_timestamp: Option<u64>,
    pub market_cap: f64,
    pub reply_count: u64,
    pub last_reply: u64,
    pub nsfw: bool,
    pub market_id: Option<u64>,
    pub inverted: Option<bool>,
    pub is_currently_live: bool,
    pub username: Option<String>,
    pub profile_image: Option<String>,
    pub usd_market_cap: f64,
}


pub async fn get_token_metadata(mint: &Pubkey) -> Result<TokenMetaData, Box<dyn Error>> {
    match reqwest::get(format!("https://frontend-api.pump.fun/coins/{:}", mint)).await {
        Ok(response) => {
            match response.status() {
                StatusCode::OK => {
                    let body = response.text().await?;
                    let data = serde_json::from_str::<TokenMetaData>(&body)?;
                    Ok(data)
                }
                _ => {
                    error!("Error retrieving PumpFun metadata: {:?}", response.status());
                    Err(format!("Error: {:?}", response.status()).into())
                }
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}
