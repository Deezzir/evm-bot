use alloy::{
    eips::BlockNumberOrTag,
    network::TransactionBuilder,
    primitives::Address,
    providers::{Provider, ReqwestProvider},
    rpc::types::TransactionRequest,
    sol,
};
use anyhow::{Context as _, Result};

sol! {
    #[sol(rpc)]
   contract ERC20 {
        function balanceOf(address owner) public view returns (uint256 balance);
        function decimals() public view returns (uint8 decimals);
        function name() public view returns (string name);
        function symbol() public view returns (string symbol);
        function transfer(address to, uint256 amount) external returns (bool success);
   }
}

#[derive(Debug, Clone, Copy)]
pub enum FeeQuote {
    Eip1559 {
        max_fee_per_gas: u128,
        max_priority_fee_per_gas: u128,
    },
    Legacy {
        gas_price: u128,
    },
}

impl FeeQuote {
    pub fn apply(self, tx: TransactionRequest) -> TransactionRequest {
        match self {
            Self::Eip1559 {
                max_fee_per_gas,
                max_priority_fee_per_gas,
            } => tx
                .with_max_fee_per_gas(max_fee_per_gas)
                .with_max_priority_fee_per_gas(max_priority_fee_per_gas),
            Self::Legacy { gas_price } => tx.with_gas_price(gas_price),
        }
    }

    pub const fn max_fee_per_gas(self) -> u128 {
        match self {
            Self::Eip1559 {
                max_fee_per_gas, ..
            } => max_fee_per_gas,
            Self::Legacy { gas_price } => gas_price,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FeeTiers {
    pub economy: FeeQuote,
    pub standard: FeeQuote,
    pub fast: FeeQuote,
    pub urgent: FeeQuote,
}

pub async fn fetch_fee_tiers(provider: &ReqwestProvider) -> Result<FeeTiers> {
    const PERCENTILES: [f64; 4] = [25.0, 50.0, 75.0, 90.0];

    if let Ok(history) = provider
        .get_fee_history(10, BlockNumberOrTag::Latest, &PERCENTILES)
        .await
        && let (Some(base_fee), Some(rewards)) = (history.next_block_base_fee(), history.reward)
    {
        let quote = |column: usize| -> Option<FeeQuote> {
            let mut samples = rewards
                .iter()
                .filter_map(|block| block.get(column).copied())
                .collect::<Vec<_>>();

            if samples.is_empty() {
                return None;
            }

            samples.sort_unstable();
            let priority_fee = samples[samples.len() / 2];

            Some(FeeQuote::Eip1559 {
                max_fee_per_gas: base_fee.saturating_mul(2).saturating_add(priority_fee),
                max_priority_fee_per_gas: priority_fee,
            })
        };

        if let (Some(economy), Some(standard), Some(fast), Some(urgent)) =
            (quote(0), quote(1), quote(2), quote(3))
        {
            return Ok(FeeTiers {
                economy,
                standard,
                fast,
                urgent,
            });
        }
    }

    let gas_price = provider.get_gas_price().await?;
    let legacy = |multiplier: u128| FeeQuote::Legacy {
        gas_price: gas_price.saturating_mul(multiplier) / 100,
    };
    Ok(FeeTiers {
        economy: legacy(90),
        standard: legacy(100),
        fast: legacy(125),
        urgent: legacy(150),
    })
}

#[derive(Debug)]
pub struct TokenMetadata {
    pub decimals: u8,
    pub name: Option<String>,
    pub symbol: Option<String>,
}

pub async fn fetch_token_metadata(
    mint: Address,
    provider: &ReqwestProvider,
) -> Result<TokenMetadata> {
    let token = ERC20::new(mint, provider);
    let decimals = token
        .decimals()
        .call()
        .await
        .context("failed to fetch token decimals")?
        .decimals;
    let name = token.name().call().await.ok().map(|result| result.name);
    let symbol = token.symbol().call().await.ok().map(|result| result.symbol);

    Ok(TokenMetadata {
        decimals,
        name,
        symbol,
    })
}
