mod chain;
mod cli;
mod commands;
mod common;
mod constants;
mod venue;

use std::time::Duration;

use alloy::providers::{ProviderBuilder, ReqwestProvider};
use anyhow::{Context as _, Ok, Result};
use clap::Parser;
use cli::{Cli, Commands};

use crate::{
    chain::ChainConfig,
    commands::{
        balance, buy_token_once, collect, collect_tokens, create_token, fund, generate,
        sell_token_once, snipe, token_balance, token_transfer, transfer,
    },
    common::Wallet,
};

struct Context<'a> {
    pub chain_config: ChainConfig,
    pub provider: ReqwestProvider,
    all_wallets: &'a [Wallet],
    http: reqwest::Client,
}

struct CommandContext<'a, 'ctx> {
    app: &'ctx Context<'a>,
    wallets: Vec<&'a Wallet>,
}

impl<'a> Context<'a> {
    fn new(cli: &cli::Cli, wallets: &'a [Wallet]) -> Result<Self> {
        let chain = cli.chain.config();
        let rpc_url = std::env::var(chain.rpc_env_key)
            .with_context(|| format!("{} environment variable not set", chain.rpc_env_key))?;
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(15))
            .build()
            .context("failed to create HTTP client")?;
        let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

        Ok(Self {
            chain_config: chain,
            all_wallets: wallets,
            provider,
            http,
        })
    }

    pub fn all(&self) -> CommandContext<'a, '_> {
        CommandContext {
            app: self,
            wallets: self.all_wallets.iter().collect(),
        }
    }

    pub fn none(&self) -> CommandContext<'a, '_> {
        CommandContext {
            app: self,
            wallets: Vec::new(),
        }
    }

    pub fn selected(
        &self,
        from: Option<usize>,
        to: Option<usize>,
        indexes: Option<&[usize]>,
    ) -> Result<CommandContext<'a, '_>> {
        let wallets = if let Some(indexes) = indexes {
            indexes
                .iter()
                .map(|&i| self.wallet(i))
                .collect::<Result<Vec<_>>>()?
        } else {
            let start = from.unwrap_or(0);
            let end = to.unwrap_or(self.all_wallets.len());

            self.all_wallets
                .get(start..end)
                .with_context(|| format!("invalid wallet range {start}..{end}"))?
                .iter()
                .collect()
        };

        Ok(CommandContext { app: self, wallets })
    }

    fn wallet(&self, index: usize) -> Result<&'a Wallet> {
        self.all_wallets.get(index).with_context(|| {
            format!(
                "wallet index {} is out of bounds ({} wallets available)",
                index,
                self.all_wallets.len()
            )
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();
    let wallets = common::read_wallets_from_csv(&cli.keys_path)?;
    let ctx = Context::new(&cli, &wallets)?;
    colored::control::set_override(!cli.no_color);

    match cli.command {
        Commands::Balance { format } => {
            let ctx = ctx.all();
            balance(&ctx, format).await
        }
        Commands::Generate {
            reserve,
            file_path,
            index,
            count,
            secrets_path,
        } => {
            let ctx = ctx.none();
            generate(
                &ctx,
                &file_path,
                count,
                index,
                reserve,
                secrets_path.as_deref(),
            )
        }
        Commands::TokenBalance { mint, format } => {
            let ctx = ctx.all();
            token_balance(&ctx, &mint, format).await
        }
        Commands::Transfer {
            amount,
            index,
            receiver,
        } => {
            let ctx = ctx.all();
            transfer(&ctx, amount, index, &receiver).await
        }
        Commands::TokenTransfer {
            mint,
            amount,
            index,
            receiver,
        } => {
            let ctx = ctx.all();
            token_transfer(&ctx, &mint, amount, index, &receiver).await
        }
        Commands::BuyTokenOnce {
            token,
            amount,
            venue,
            index,
        } => {
            let ctx = ctx.all();
            let deployment = cli.chain.trading(venue.into())?;
            buy_token_once(&ctx, deployment, &token, amount, index).await
        }
        Commands::SellTokenOnce {
            amount,
            token,
            index,
            venue,
        } => {
            let ctx = ctx.all();
            let deployment = cli.chain.trading(venue.into())?;
            sell_token_once(&ctx, deployment, &token, amount, index).await
        }
        Commands::Fund {
            amount,
            sender_index,
            from,
            to,
            indexes,
            random,
        } => {
            let ctx = ctx.selected(from, to, indexes.as_deref())?;
            fund(&ctx, amount, sender_index, random).await
        }
        Commands::Collect {
            receiver,
            from,
            to,
            indexes,
        } => {
            let ctx = ctx.selected(from, to, indexes.as_deref())?;
            collect(&ctx, &receiver).await
        }
        Commands::CollectTokens {
            mint,
            receiver,
            from,
            to,
            indexes,
        } => {
            let ctx = ctx.selected(from, to, indexes.as_deref())?;
            collect_tokens(&ctx, &mint, &receiver).await
        }
        Commands::CreateToken {
            name,
            symbol,
            metadata_uri,
            launchpad,
            index,
        } => {
            let ctx = ctx.all();
            let deployment: chain::LaunchpadDeployment = cli.chain.launchpad(launchpad.into())?;
            create_token(&ctx, deployment, &name, &symbol, &metadata_uri, index).await
        }
        Commands::Snipe { launchpad } => {
            let ctx = ctx.all();
            let deployment = cli.chain.launchpad(launchpad.into())?;
            snipe(&ctx, deployment).await
        }
    }
}
