use crate::{
    Context,
    chain::{LaunchpadDeployment, TradingDeployment},
    cli::OutputFormat,
};

pub fn generate(
    ctx: &Context,
    file_path: &String,
    count: usize,
    index: usize,
    reserve: bool,
    secrets_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generate");
    Ok(())
}

pub async fn balance(
    ctx: &Context,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Balance");
    Ok(())
}

pub async fn token_balance(
    ctx: &Context,
    mint: &String,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("TokenBalance");
    Ok(())
}

pub async fn transfer(
    ctx: &Context,
    amount: f64,
    index: usize,
    receiver: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Transfer");
    Ok(())
}

pub async fn token_transfer(
    ctx: &Context,
    mint: &String,
    amount: f64,
    index: usize,
    receiver: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("TokenTransfer");
    Ok(())
}

pub async fn buy(
    ctx: &Context,
    deployment: TradingDeployment,
    token: &String,
    amount: f64,
    index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Buy");
    Ok(())
}

pub async fn create_token(
    ctx: &Context,
    deployment: LaunchpadDeployment,
    name: String,
    symbol: String,
    metadata_uri: String,
    index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("CreateToken");
    Ok(())
}

pub async fn snipe(
    ctx: &Context,
    deployment: LaunchpadDeployment,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Snipe");
    Ok(())
}
