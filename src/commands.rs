use crate::{
    CommandContext,
    chain::{LaunchpadDeployment, TradingDeployment},
    cli::OutputFormat,
    common::{
        TableAlignment, format_currency, format_table_footer, format_table_header,
        format_table_row, to_token_units, to_wei,
    },
    constants::TABLE_COLUMN_WIDTHS,
    price::{CryptoPrices, fetch_collateral_prices, fetch_token_price},
    trade::{fetch_fee_tiers, fetch_token_metadata},
    wallet::{Wallet, write_wallets_to_csv},
};
use alloy::{
    primitives::{U256, utils::format_units},
    signers::local::PrivateKeySigner,
};
use anyhow::{Context as _, Result};
use colored::Colorize;
use futures::future::try_join_all;
use std::{fs, path::Path};

pub fn generate(
    _ctx: &CommandContext<'_, '_>,
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
    let prices = match fetch_collateral_prices(&ctx.app.http).await {
        Ok(prices) => prices,
        Err(error) => {
            eprintln!("Failed to fetch prices: {error:#}");
            CryptoPrices {
                bnb: 0.0,
                eth: 0.0,
                hype: 0.0,
                pol: 0.0,
            }
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
            println!("{}", "Getting the balance of the wallets...".yellow());
            println!(
                "{}\n",
                format!("Wallet Count: {}", ctx.wallets.len()).yellow()
            );

            let header_columns = vec![
                crate::common::TableColumn::new("Id", TABLE_COLUMN_WIDTHS.id),
                crate::common::TableColumn::new("Name", TABLE_COLUMN_WIDTHS.name),
                crate::common::TableColumn::new("Public Key", TABLE_COLUMN_WIDTHS.public_key)
                    .with_alignment(TableAlignment::Center),
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

                let balance = format_units(*wei, "eth")?
                    .parse::<f64>()
                    .context("failed to convert native balance")?;
                let usd_balance = balance * prices.get_price(collateral);

                println!(
                    "{}",
                    format_table_row(&[
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

            let total = format_units(total_wei, "eth")?
                .parse::<f64>()
                .context("failed to convert total balance")?;
            let total_usd = total * prices.get_price(collateral);

            println!("{}", format_table_footer(&header_columns));

            println!(
                "\nTotal balance: {} {}",
                format_currency(total).bold(),
                collateral.name()
            );
            println!("Total USD value: ${}\n", format_currency(total_usd).bold());
        }
        OutputFormat::Json => {
            println!("JSON not supported yet");
        }
        OutputFormat::Csv => {
            println!(
                "id,name,pubkey,{}_balance,usd_balance",
                collateral.name().to_lowercase()
            );
            for (i, (wallet, wei)) in ctx.wallets.iter().zip(&balances).enumerate() {
                let balance = format_units(*wei, "eth")?
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
    ctx: &CommandContext<'_, '_>,
    mint: &str,
    format: OutputFormat,
) -> Result<()> {
    if ctx.wallets.is_empty() {
        println!("{}", "No wallets found".yellow());
        return Ok(());
    }

    let mint = mint
        .parse::<alloy::primitives::Address>()
        .context("invalid address provided")?;
    let mint_asset = fetch_token_metadata(mint, &ctx.app.provider).await?;
    let balances = try_join_all(
        ctx.wallets
            .iter()
            .map(|wallet| wallet.get_token_balance(&ctx.app.provider, mint)),
    )
    .await?;
    let mut total_balance = U256::ZERO;
    let token_name = mint_asset.name.as_deref().unwrap_or("UNKNOWN");
    let token_symbol = mint_asset.symbol.as_deref().unwrap_or("UNKNOWN");
    let token_decimals = mint_asset.decimals;
    let token_price =
        fetch_token_price(&mint.to_string(), ctx.app.chain_config.chain, &ctx.app.http)
            .await?
            .unwrap_or(0.0);

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                format!(
                    "Getting the token balance of the wallets by the mint {}...",
                    mint
                )
                .yellow()
            );
            println!(
                "{}",
                format!("Token: {} | Symbol: {}", token_name, token_symbol).yellow()
            );
            println!(
                "{}\n",
                format!("Wallet Count: {}", ctx.wallets.len()).yellow()
            );

            let header_columns = vec![
                crate::common::TableColumn::new("Id", TABLE_COLUMN_WIDTHS.id),
                crate::common::TableColumn::new("Name", TABLE_COLUMN_WIDTHS.name),
                crate::common::TableColumn::new("Public Key", TABLE_COLUMN_WIDTHS.public_key)
                    .with_alignment(TableAlignment::Center),
                crate::common::TableColumn::new(
                    format!("{} Balance", token_symbol),
                    TABLE_COLUMN_WIDTHS.token_balance,
                )
                .with_alignment(TableAlignment::Right),
                crate::common::TableColumn::new("USD Value", TABLE_COLUMN_WIDTHS.usd_balance)
                    .with_alignment(TableAlignment::Right),
            ];

            println!("{}", format_table_header(&header_columns));

            for (i, (wallet, token_balance)) in ctx.wallets.iter().zip(&balances).enumerate() {
                total_balance += token_balance;

                let balance = format_units(*token_balance, token_decimals)?
                    .parse::<f64>()
                    .context("failed to convert native balance")?;
                let usd_balance = balance * token_price;

                println!(
                    "{}",
                    format_table_row(&[
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
                            TABLE_COLUMN_WIDTHS.token_balance
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

            let total = format_units(total_balance, token_decimals)?
                .parse::<f64>()
                .context("failed to convert total balance")?;
            let total_usd = total * token_price;

            println!("{}", format_table_footer(&header_columns));

            println!(
                "\nTotal balance: {} {}",
                format_currency(total).bold(),
                token_symbol,
            );
            println!("Total USD value: ${}\n", format_currency(total_usd).bold());
        }
        OutputFormat::Json => {
            println!("JSON not supported yet");
        }
        OutputFormat::Csv => {
            println!("id,name,pubkey,{}_balance", token_symbol.to_lowercase());
            for (i, (wallet, tokens)) in ctx.wallets.iter().zip(&balances).enumerate() {
                let balance = format_units(*tokens, token_decimals)?
                    .parse::<f64>()
                    .context("failed to convert balance")?;

                println!(
                    "{},{},{},{}",
                    i,
                    wallet.name,
                    wallet.keypair.address(),
                    balance,
                );
            }
        }
    }

    Ok(())
}

pub async fn transfer(
    ctx: &CommandContext<'_, '_>,
    amount: &str,
    index: usize,
    receiver: &str,
) -> Result<()> {
    if amount.trim_start().starts_with('-') {
        anyhow::bail!("transfer amount must be greater than zero");
    }

    let sender = ctx.wallet(index)?;
    let receiver = receiver
        .parse::<alloy::primitives::Address>()
        .context("invalid address provided")?;
    let collateral = ctx.app.chain_config.chain.collateral().unwrap();

    if sender.keypair.address() == receiver {
        anyhow::bail!("Sender and receiver addresses cannot be the same");
    }

    println!(
        "{}",
        format!(
            "Transferring {} {} from {} to {}...",
            amount,
            collateral.name(),
            sender.keypair.address(),
            receiver
        )
        .yellow()
    );

    let amount_wei = to_wei(amount)?;
    if amount_wei == U256::ZERO {
        anyhow::bail!("transfer amount must be greater than zero");
    }
    let balance = sender.get_balance(&ctx.app.provider).await?;
    let fee_tiers = fetch_fee_tiers(&ctx.app.provider).await?;
    if balance == U256::ZERO {
        anyhow::bail!("Sender has no balance");
    }

    let estimated_fee = sender
        .estimate_transfer_fee(&ctx.app.provider, receiver, amount_wei, fee_tiers.standard)
        .await?;
    let required_balance = amount_wei
        .checked_add(estimated_fee)
        .context("transfer amount and fee overflow")?;

    if balance < required_balance {
        anyhow::bail!(
            "Sender balance is not enough to transfer {amount} {}",
            collateral.name()
        );
    }

    let receipt = sender
        .transfer_wei(
            ctx.app.rpc_url.clone(),
            receiver,
            amount_wei,
            fee_tiers.standard,
        )
        .await?;
    println!(
        "{}",
        format!(
            "Transaction completed, signature: {}",
            receipt.transaction_hash
        )
        .green()
    );

    Ok(())
}

pub async fn token_transfer(
    ctx: &CommandContext<'_, '_>,
    mint: &str,
    amount: &str,
    index: usize,
    receiver: &str,
) -> Result<()> {
    if amount.trim_start().starts_with('-') {
        anyhow::bail!("transfer amount must be greater than zero");
    }

    let sender = ctx.wallet(index)?;
    let receiver = receiver
        .parse::<alloy::primitives::Address>()
        .context("invalid address provided")?;

    if sender.keypair.address() == receiver {
        anyhow::bail!("Sender and receiver addresses cannot be the same");
    }

    let mint = mint
        .parse::<alloy::primitives::Address>()
        .context("invalid address provided")?;
    let fee_tiers = fetch_fee_tiers(&ctx.app.provider).await?;
    let mint_asset = fetch_token_metadata(mint, &ctx.app.provider).await?;
    let token_name = mint_asset.name.as_deref().unwrap_or("UNKNOWN");
    let token_symbol = mint_asset.symbol.as_deref().unwrap_or("UNKNOWN");
    let token_decimals = mint_asset.decimals;

    println!(
        "{}",
        format!(
            "Transferring {} {} from {} to {}...",
            amount,
            token_symbol,
            sender.keypair.address(),
            receiver
        )
        .yellow()
    );

    let amount_tokens = to_token_units(amount, token_decimals)?;
    if amount_tokens == U256::ZERO {
        anyhow::bail!("transfer amount must be greater than zero");
    }
    let token_balance = sender.get_token_balance(&ctx.app.provider, mint).await?;

    if token_balance == U256::ZERO {
        anyhow::bail!("Sender has no token balance for {token_name}",);
    }
    if token_balance < amount_tokens {
        anyhow::bail!("Sender balance is not enough to transfer {amount} {token_symbol}",);
    }

    let receipt = sender
        .transfer_token(
            ctx.app.rpc_url.clone(),
            mint,
            receiver,
            amount_tokens,
            fee_tiers.standard,
        )
        .await?;
    println!(
        "{}",
        format!(
            "Transaction completed, signature: {}",
            receipt.transaction_hash
        )
        .green()
    );

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
