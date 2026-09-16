use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    chain::CollateralToken,
    constants::{DEFILLAMA_PRICE_URL, DEXSCREENER_PRICE_URL},
};

#[derive(Debug, Deserialize)]
struct DefiLlamaResponse {
    coins: HashMap<String, DefiLlamaCoin>,
}

#[derive(Debug, Deserialize)]
struct DefiLlamaCoin {
    price: Option<f64>,
}

#[derive(Debug)]
pub struct CryptoPrices {
    pub bnb: f64,
    pub eth: f64,
    pub hype: f64,
    pub pol: f64,
}

impl CryptoPrices {
    pub fn get_price(&self, token: CollateralToken) -> f64 {
        match token {
            CollateralToken::Bnb => self.bnb,
            CollateralToken::Eth => self.eth,
            CollateralToken::Hype => self.hype,
            CollateralToken::Usdc => 1.0,
            CollateralToken::Pol => self.pol,
        }
    }
}

pub async fn fetch_collateral_prices(client: &reqwest::Client) -> Result<CryptoPrices> {
    const ETH: &str = "coingecko:ethereum";
    const BNB: &str = "coingecko:binancecoin";
    const HYPE: &str = "coingecko:hyperliquid";
    const POL: &str = "coingecko:polygon-ecosystem-token";

    let response = client
        .get(format!("{DEFILLAMA_PRICE_URL}/{ETH},{BNB},{HYPE},{POL}"))
        .send()
        .await
        .context("failed to fetch collateral prices")?
        .error_for_status()
        .context("DefiLlama returned an error")?
        .json::<DefiLlamaResponse>()
        .await
        .context("failed to parse DefiLlama collateral response")?;

    let price = |coin_id: &str| {
        response
            .coins
            .get(coin_id)
            .and_then(|coin| coin.price)
            .with_context(|| format!("{coin_id} price not found"))
    };

    Ok(CryptoPrices {
        bnb: price(BNB)?,
        eth: price(ETH)?,
        hype: price(HYPE)?,
        pol: price(POL)?,
    })
}

#[derive(Debug, Deserialize)]
struct DexScreenerResponse {
    pairs: Option<Vec<DexScreenerPair>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DexScreenerPair {
    chain_id: String,
    price_usd: Option<String>,
    liquidity: Option<DexScreenerLiquidity>,
}

#[derive(Debug, Deserialize)]
struct DexScreenerLiquidity {
    usd: Option<f64>,
}

fn dexscreener_chain(chain: crate::chain::Chain) -> Option<&'static str> {
    match chain {
        crate::chain::Chain::Ethereum => Some("ethereum"),
        crate::chain::Chain::Bnb => Some("bsc"),
        crate::chain::Chain::Robinhood => Some("robinhood"),
        crate::chain::Chain::Base => Some("base"),
        crate::chain::Chain::Ink => Some("ink"),
        crate::chain::Chain::Hyper => Some("hyperevm"),
        crate::chain::Chain::Arc => None,
        crate::chain::Chain::Polygon => Some("polygon"),
    }
}

fn defillama_chain(chain: crate::chain::Chain) -> Option<&'static str> {
    match chain {
        crate::chain::Chain::Ethereum => Some("ethereum"),
        crate::chain::Chain::Bnb => Some("bsc"),
        crate::chain::Chain::Robinhood => None,
        crate::chain::Chain::Base => Some("base"),
        crate::chain::Chain::Ink => None,
        crate::chain::Chain::Hyper => Some("hyperliquid"),
        crate::chain::Chain::Arc => None,
        crate::chain::Chain::Polygon => Some("polygon"),
    }
}

async fn fetch_dexscreener_price(
    mint: &str,
    chain: crate::chain::Chain,
    client: &reqwest::Client,
) -> Result<Option<f64>> {
    let Some(chain_id) = dexscreener_chain(chain) else {
        return Ok(None);
    };

    let response = client
        .get(format!("{DEXSCREENER_PRICE_URL}/{mint}"))
        .send()
        .await?
        .error_for_status()?
        .json::<DexScreenerResponse>()
        .await?;

    let mut candidates = response
        .pairs
        .unwrap_or_default()
        .into_iter()
        .filter_map(|pair| {
            if pair.chain_id != chain_id {
                return None;
            }

            let price = pair.price_usd?.parse::<f64>().ok()?;
            let liquidity = pair
                .liquidity
                .and_then(|liquidity| liquidity.usd)
                .unwrap_or(0.0);

            Some((liquidity, price))
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| right.0.total_cmp(&left.0));

    Ok(candidates.first().map(|(_, price)| *price))
}

async fn fetch_defillama_price(
    mint: &str,
    chain: crate::chain::Chain,
    client: &reqwest::Client,
) -> Result<Option<f64>> {
    let Some(chain_id) = defillama_chain(chain) else {
        return Ok(None);
    };

    let coin_id = format!("{chain_id}:{}", mint.to_lowercase());

    let response = client
        .get(format!("{DEFILLAMA_PRICE_URL}/{coin_id}"))
        .send()
        .await?
        .error_for_status()?
        .json::<DefiLlamaResponse>()
        .await?;

    Ok(response.coins.get(&coin_id).and_then(|coin| coin.price))
}

pub async fn fetch_token_price(
    mint: &str,
    chain: crate::chain::Chain,
    client: &reqwest::Client,
) -> Result<Option<f64>> {
    if let Ok(Some(price)) = fetch_defillama_price(mint, chain, client).await {
        return Ok(Some(price));
    }

    match fetch_dexscreener_price(mint, chain, client).await {
        Ok(price) => Ok(price),
        Err(_) => Ok(None),
    }
}
