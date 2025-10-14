mod chain;
mod cli;
mod commands;
mod constants;
mod venue;

use clap::Parser;
use cli::{CLI, Commands};

use crate::{
    chain::ChainConfig,
    commands::{
        balance, buy, create_token, generate, snipe, token_balance, token_transfer, transfer,
    },
};

struct Context {
    pub chain: ChainConfig,
    pub rpc_url: String,
}

impl Context {
    fn new(cli: &cli::CLI) -> Result<Self, Box<dyn std::error::Error>> {
        let chain = cli.chain.config();
        let rpc_url = std::env::var(chain.rpc_env_key)
            .map_err(|_| format!("{} environment variable not set", chain.rpc_env_key))?;

        Ok(Self { chain, rpc_url })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let cli = CLI::parse();
    let ctx = Context::new(&cli)?;

    match cli.command {
        Commands::Balance { format } => balance(&ctx, format).await,
        Commands::Generate {
            reserve,
            file_path,
            index,
            count,
            secrets_path: keys,
        } => generate(&ctx, &file_path, count, index, reserve, keys),
        Commands::TokenBalance { mint, format } => token_balance(&ctx, &mint, format).await,
        Commands::Transfer {
            amount,
            index,
            receiver,
        } => transfer(&ctx, amount, index, &receiver).await,
        Commands::TokenTransfer {
            mint,
            amount,
            index,
            receiver,
        } => token_transfer(&ctx, &mint, amount, index, &receiver).await,
        Commands::Buy {
            token,
            amount,
            venue,
            index,
        } => {
            let deployment = cli.chain.trading(venue.into())?;
            buy(&ctx, deployment, &token, amount, index).await
        }
        Commands::CreateToken {
            name,
            symbol,
            metadata_uri,
            launchpad,
            index,
        } => {
            let deployment: chain::LaunchpadDeployment = cli.chain.launchpad(launchpad.into())?;
            create_token(&ctx, deployment, name, symbol, metadata_uri, index).await
        }
        Commands::Snipe { launchpad } => {
            let deployment = cli.chain.launchpad(launchpad.into())?;
            snipe(&ctx, deployment).await
        }
    }
}
