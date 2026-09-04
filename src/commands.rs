use std::{fs, path::Path};

use crate::{
    CommandContext,
    chain::{LaunchpadDeployment, TradingDeployment},
    cli::OutputFormat,
    common::{
        CryptoPrices, TABLE_COLUMN_WIDTHS, TableAlignment, Wallet, fetch_crypto_prices,
        format_currency, format_table_footer, format_table_header, format_table_row,
        write_wallets_to_csv,
    },
};
use alloy::{
    primitives::{U256, utils::format_units},
    signers::local::PrivateKeySigner,
};
use anyhow::{Context as _, Result};
use colored::Colorize;
use futures::future::try_join_all;

pub fn generate(
    _: &CommandContext<'_, '_>,
    file_path: &Path,
    count: usize,
    index: Option<usize>,
    create_reserve: bool,
    secrets_path: Option<&Path>,
) -> Result<()> {
    println!(
        "{}",
        format!(
            "Generating {} keypairs...",
            count + usize::from(create_reserve)
        )
        .yellow()
    );

    let starting_index = index.unwrap_or(1);
    let mut wallets = Vec::new();

    if let Some(path) = secrets_path {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read secrets file {}", path.display()))?;

        for (i, line) in content.lines().enumerate() {
            let private_key = line.trim();
            if private_key.is_empty() {
                continue;
            }

            let signer = private_key.parse::<PrivateKeySigner>().with_context(|| {
                format!(
                    "invalid private key in {} at line {}",
                    path.display(),
                    i + 1
                )
            })?;

            let wallet_index = starting_index + wallets.len() - usize::from(create_reserve);

            wallets.push(Wallet::from_keypair(
                format!("wallet[{wallet_index}]"),
                signer,
                false,
            ));
        }
    }

    if create_reserve {
        wallets.push(Wallet::new("reserve".to_string(), true));
    }

    for i in 0..count {
        wallets.push(Wallet::new(format!("wallet[{}]", i + starting_index), None));
    }

    write_wallets_to_csv(file_path, &wallets)?;

    println!("{}", "Wallet generation completed".green());
    Ok(())
}

pub async fn balance(ctx: &CommandContext<'_, '_>, format: OutputFormat) -> Result<()> {
    if ctx.wallets.is_empty() {
        println!("{}", "No wallets found".yellow());
        return Ok(());
    }

    let mut total_wei = U256::ZERO;
    let collateral = ctx.app.chain_config.chain.collateral().unwrap();
    let prices = match fetch_crypto_prices(&ctx.app.http).await {
        Ok(prices) => prices,
        Err(error) => {
            eprintln!("Failed to fetch prices: {error:#}");
            CryptoPrices { bnb: 0.0, eth: 0.0 }
        }
    };
    let balances = try_join_all(
        ctx.wallets
            .iter()
            .map(|wallet| wallet.get_balance(&ctx.app.provider)),
    )
    .await?;

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                format!("Wallet Count: {}", ctx.wallets.len()).yellow()
            );
            println!("{}\n", "Getting the balance of the wallets...".yellow());

            let header_columns = vec![
                crate::common::TableColumn::new("Id", TABLE_COLUMN_WIDTHS.id),
                crate::common::TableColumn::new("Name", TABLE_COLUMN_WIDTHS.name),
                crate::common::TableColumn::new("Public Key", TABLE_COLUMN_WIDTHS.public_key),
                crate::common::TableColumn::new(
                    format!("{} Balance", collateral.name()),
                    TABLE_COLUMN_WIDTHS.collateral_balance,
                )
                .with_alignment(TableAlignment::Right),
                crate::common::TableColumn::new("USD Value", TABLE_COLUMN_WIDTHS.usd_balance)
                    .with_alignment(TableAlignment::Right),
            ];

            println!("{}", format_table_header(&header_columns));

            for (i, (wallet, wei)) in ctx.wallets.iter().zip(&balances).enumerate() {
                total_wei += wei;

                let balance = format_units(*wei, 18)?
                    .parse::<f64>()
                    .context("failed to convert native balance")?;
                let usd_balance = balance * prices.get_price(collateral);

                println!(
                    "{}",
                    format_table_row(&vec![
                        crate::common::TableColumn::new(i.to_string(), TABLE_COLUMN_WIDTHS.id),
                        crate::common::TableColumn::new(
                            format!(
                                "{}{}",
                                wallet.name.clone(),
                                if wallet.is_reserve { "*" } else { "" }
                            ),
                            TABLE_COLUMN_WIDTHS.name
                        ),
                        crate::common::TableColumn::new(
                            wallet.keypair.address().to_string(),
                            TABLE_COLUMN_WIDTHS.public_key
                        ),
                        crate::common::TableColumn::new(
                            balance.to_string(),
                            TABLE_COLUMN_WIDTHS.collateral_balance
                        )
                        .with_alignment(TableAlignment::Right),
                        crate::common::TableColumn::new(
                            format_currency(usd_balance),
                            TABLE_COLUMN_WIDTHS.usd_balance
                        )
                        .with_alignment(TableAlignment::Right),
                    ])
                );
            }

            let total = format_units(total_wei, 18)?
                .parse::<f64>()
                .context("failed to convert total balance")?;
            let total_usd = total * prices.get_price(collateral);

            println!("{}", format_table_footer(&header_columns));

            println!(
                "\n{}",
                format!(
                    "Total balance: {} {}",
                    format_currency(total).bold(),
                    collateral.name()
                )
            );
            println!(
                "{}\n",
                format!("Total USD value: ${}", format_currency(total_usd).bold(),)
            );
        }
        OutputFormat::Json => {
            println!("JSON not supported yet");
        }
        OutputFormat::Csv => {
            println!(
                "{}",
                format!("id,name,pubkey,{}_balance,usd_balance", collateral.name())
            );
            for (i, (wallet, wei)) in ctx.wallets.iter().zip(&balances).enumerate() {
                let balance = format_units(*wei, 18)?
                    .parse::<f64>()
                    .context("failed to convert native balance")?;
                let usd_balance = balance * prices.get_price(collateral);

                println!(
                    "{},{},{},{},{}",
                    i,
                    wallet.name,
                    wallet.keypair.address(),
                    balance,
                    usd_balance
                );
            }
        }
    }

    Ok(())
}

pub async fn token_balance(
    _ctx: &CommandContext<'_, '_>,
    _mint: &str,
    _format: OutputFormat,
) -> Result<()> {
    println!("TokenBalance");
    Ok(())
}

pub async fn transfer(
    _ctx: &CommandContext<'_, '_>,
    _amount: f64,
    _index: usize,
    _receiver: &str,
) -> Result<()> {
    println!("Transfer");
    Ok(())
}

pub async fn token_transfer(
    _ctx: &CommandContext<'_, '_>,
    _mint: &str,
    _amount: f64,
    _index: usize,
    _receiver: &str,
) -> Result<()> {
    println!("TokenTransfer");
    Ok(())
}

pub async fn buy_token_once(
    _ctx: &CommandContext<'_, '_>,
    _deployment: TradingDeployment,
    _token: &str,
    _amount: f64,
    _index: usize,
) -> Result<()> {
    println!("Buy");
    Ok(())
}

pub async fn sell_token_once(
    _ctx: &CommandContext<'_, '_>,
    _deployment: TradingDeployment,
    _token: &str,
    _amount: f64,
    _index: usize,
) -> Result<()> {
    println!("Sell");
    Ok(())
}

pub async fn collect(_ctx: &CommandContext<'_, '_>, _receiver: &str) -> Result<()> {
    println!("Collect");
    Ok(())
}

pub async fn fund(
    _ctx: &CommandContext<'_, '_>,
    _amount: f64,
    _sender_index: usize,
    _random: Option<bool>,
) -> Result<()> {
    println!("Fund");
    Ok(())
}

pub async fn collect_tokens(
    _ctx: &CommandContext<'_, '_>,
    _mint: &str,
    _receiver: &str,
) -> Result<()> {
    println!("CollectTokens");
    Ok(())
}

pub async fn create_token(
    _ctx: &CommandContext<'_, '_>,
    _deployment: LaunchpadDeployment,
    _name: &str,
    _symbol: &str,
    _metadata_uri: &str,
    _index: usize,
) -> Result<()> {
    println!("CreateToken");
    Ok(())
}

pub async fn snipe(_ctx: &CommandContext<'_, '_>, _deployment: LaunchpadDeployment) -> Result<()> {
    println!("Snipe");
    Ok(())
}
