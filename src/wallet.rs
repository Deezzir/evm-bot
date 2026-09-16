use alloy::{
    hex,
    network::{EthereumWallet, TransactionBuilder},
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder, ReqwestProvider},
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::local::PrivateKeySigner,
};
use anyhow::{Context as _, Ok, Result, bail};
use reqwest::Url;
use std::{
    fs::{File, OpenOptions},
    io::ErrorKind,
    path::Path,
};

use crate::{
    constants::TX_REQUIRED_CONFIRMATIONS,
    trade::{ERC20, FeeQuote},
};

#[derive(Debug)]
pub struct Wallet {
    pub name: String,
    pub keypair: PrivateKeySigner,
    pub is_reserve: bool,
}

impl Wallet {
    pub fn new(name: impl Into<Option<String>>, is_reserve: impl Into<Option<bool>>) -> Self {
        Self {
            name: name.into().unwrap_or("wallet".into()),
            keypair: PrivateKeySigner::random(),
            is_reserve: is_reserve.into().unwrap_or(false),
        }
    }

    pub fn from_keypair(
        name: impl Into<Option<String>>,
        keypair: PrivateKeySigner,
        is_reserve: bool,
    ) -> Self {
        Self {
            name: name.into().unwrap_or("wallet".into()),
            keypair,
            is_reserve,
        }
    }

    pub fn csv_header() -> [&'static str; 4] {
        ["name", "private_key", "is_reserve", "public_key"]
    }

    pub fn csv_row(&self) -> [String; 4] {
        [
            self.name.clone(),
            format!("0x{}", hex::encode(self.keypair.to_bytes())),
            self.is_reserve.to_string(),
            self.keypair.address().to_string(),
        ]
    }

    pub async fn get_balance(&self, provider: &ReqwestProvider) -> Result<U256> {
        provider
            .get_balance(self.keypair.address())
            .await
            .with_context(|| {
                format!(
                    "failed to get balance for wallet {} ({})",
                    self.name,
                    self.keypair.address()
                )
            })
    }
    pub async fn get_token_balance(
        &self,
        provider: &ReqwestProvider,
        mint: Address,
    ) -> Result<U256> {
        let result = ERC20::new(mint, provider)
            .balanceOf(self.keypair.address())
            .call()
            .await?;

        Ok(result.balance)
    }

    pub async fn transfer_wei(
        &self,
        rpc_url: Url,
        receiver: Address,
        amount: U256,
        fee: FeeQuote,
    ) -> Result<TransactionReceipt> {
        let wallet = EthereumWallet::from(self.keypair.clone());

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(rpc_url);

        let tx = fee.apply(
            TransactionRequest::default()
                .with_to(receiver)
                .with_value(amount),
        );

        let pending = provider.send_transaction(tx).await?;
        let receipt = pending
            .with_required_confirmations(TX_REQUIRED_CONFIRMATIONS)
            .get_receipt()
            .await?;

        if !receipt.status() {
            bail!("transaction reverted: {:?}", receipt.transaction_hash);
        }

        Ok(receipt)
    }

    pub async fn estimate_transfer_fee(
        &self,
        provider: &ReqwestProvider,
        receiver: Address,
        amount: U256,
        fee: FeeQuote,
    ) -> Result<U256> {
        let tx = fee.apply(
            TransactionRequest::default()
                .with_from(self.keypair.address())
                .with_to(receiver)
                .with_value(amount),
        );
        let gas = provider
            .estimate_gas(&tx)
            .await
            .context("failed to estimate transfer gas")?;

        Ok(U256::from(gas).saturating_mul(U256::from(fee.max_fee_per_gas())))
    }

    pub async fn transfer_token(
        &self,
        rpc_url: Url,
        token: Address,
        receiver: Address,
        amount: U256,
        fee: FeeQuote,
    ) -> Result<TransactionReceipt> {
        let wallet = EthereumWallet::from(self.keypair.clone());

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(rpc_url);

        let token_contract = ERC20::new(token, &provider);
        let transfer = token_contract
            .transfer(receiver, amount)
            .from(self.keypair.address());
        if !transfer.call().await?.success {
            bail!("token transfer returned false");
        }

        let tx = fee.apply(
            TransactionRequest::default()
                .with_to(token)
                .with_value(U256::ZERO)
                .with_input(transfer.calldata().clone()),
        );

        let pending = provider.send_transaction(tx).await?;
        let receipt = pending
            .with_required_confirmations(TX_REQUIRED_CONFIRMATIONS)
            .get_receipt()
            .await?;

        if !receipt.status() {
            bail!("transaction reverted: {:?}", receipt.transaction_hash);
        }

        Ok(receipt)
    }
}

pub fn write_wallets_to_csv(path: impl AsRef<Path>, wallets: &[Wallet]) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }
    let is_new_file = !path.exists()
        || path
            .metadata()
            .map(|metadata| metadata.len() == 0)
            .unwrap_or(false);

    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.mode(0o600);
    }
    let file = options
        .open(path)
        .with_context(|| format!("failed to open or create {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("failed to secure {}", path.display()))?;
    }

    let mut writer: csv::Writer<File> = csv::Writer::from_writer(file);
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    if is_new_file {
        let mut header = Wallet::csv_header().to_vec();
        header.push("created_at");
        writer
            .write_record(&header)
            .with_context(|| format!("failed to write CSV header to {}", path.display()))?;
    }

    for wallet in wallets {
        let mut row = wallet.csv_row().to_vec();
        row.push(date.clone());
        writer
            .write_record(row)
            .with_context(|| format!("failed to write wallet to {}", path.display()))?;
    }
    writer
        .flush()
        .with_context(|| format!("failed to flush {}", path.display()))?;
    Ok(())
}

pub fn read_wallets_from_csv(path: impl AsRef<Path>) -> Result<Vec<Wallet>> {
    let path = path.as_ref();
    let file = match File::open(path) {
        std::result::Result::Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(error) => {
            bail!("failed to open {}: {}", path.display(), error);
        }
    };

    let mut reader = csv::Reader::from_reader(file);
    let mut wallets = Vec::new();

    let mut expected = Wallet::csv_header().to_vec();
    expected.push("created_at");
    let actual = reader
        .headers()
        .with_context(|| format!("failed to read CSV header from {}", path.display()))?;

    if actual != expected.as_slice() {
        bail!("unexpected CSV header in {}", path.display());
    }

    let mut next_reserve_insert_index = 0;
    for (i, result) in reader.records().enumerate() {
        let record =
            result.with_context(|| format!("failed to read CSV record at line {}", i + 2))?;

        let name = record
            .get(0)
            .with_context(|| format!("missing wallet name at line {}", i + 2))?
            .to_owned();
        let keypair = record
            .get(1)
            .with_context(|| format!("missing private key at line {}", i + 2))?
            .parse::<PrivateKeySigner>()
            .with_context(|| format!("invalid private key at line {}", i + 2))?;
        let is_reserve = record
            .get(2)
            .with_context(|| format!("missing reserve flag at line {}", i + 2))?
            .parse::<bool>()
            .with_context(|| format!("invalid reserve flag at line {}", i + 2))?;

        let wallet = Wallet::from_keypair(name, keypair, is_reserve);
        if is_reserve {
            wallets.insert(next_reserve_insert_index, wallet);
            next_reserve_insert_index += 1;
        } else {
            wallets.push(wallet);
        }
    }

    Ok(wallets)
}
