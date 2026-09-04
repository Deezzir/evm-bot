use num_format::{Locale, ToFormattedString};
use std::{
    fs::{File, OpenOptions},
    io::ErrorKind,
    path::Path,
};

use alloy::{
    hex,
    primitives::U256,
    providers::{Provider, ReqwestProvider},
    signers::local::PrivateKeySigner,
};
use anyhow::{Context as _, Ok, Result, bail};
use serde::Deserialize;

use crate::{chain::CollateralToken, constants::PRICE_URL};

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

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open or create {}", path.display()))?;

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
            return Err(error).with_context(|| format!("failed to open {}", path.display()));
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

#[derive(Debug, Deserialize)]
struct BinancePrice {
    symbol: String,
    price: String,
}

#[derive(Debug)]
pub struct CryptoPrices {
    pub bnb: f64,
    pub eth: f64,
}

impl CryptoPrices {
    pub fn get_price(&self, token: CollateralToken) -> f64 {
        match token {
            CollateralToken::Bnb => self.bnb,
            CollateralToken::Eth => self.eth,
        }
    }
}

pub async fn fetch_crypto_prices(client: &reqwest::Client) -> Result<CryptoPrices> {
    let response = client
        .get(PRICE_URL)
        .query(&[("symbols", r#"["ETHUSDT","BNBUSDT"]"#)])
        .send()
        .await
        .context("failed to fetch crypto prices")?
        .error_for_status()
        .context("Binance returned an error")?
        .json::<Vec<BinancePrice>>()
        .await
        .context("failed to parse Binance response")?;

    let mut bnb = None;
    let mut eth = None;

    for item in response {
        let price = item
            .price
            .parse::<f64>()
            .with_context(|| format!("invalid price for {}", item.symbol))?;

        match item.symbol.as_str() {
            "BNBUSDT" => bnb = Some(price),
            "ETHUSDT" => eth = Some(price),
            _ => {}
        }
    }

    Ok(CryptoPrices {
        bnb: bnb.context("BNBUSDT price not found")?,
        eth: eth.context("ETHUSDT price not found")?,
    })
}

pub fn format_currency(value: f64) -> String {
    let int_part = value.trunc() as i64;
    let frac_part = (value.fract().abs() * 100.0).round() as u64;

    format!(
        "{}.{:02}",
        int_part.to_formatted_string(&Locale::en),
        frac_part
    )
}

#[derive(Clone, Copy)]
pub enum TableAlignment {
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy)]
pub enum TableBorderChar {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Horizontal,
    Vertical,
    Middle,
    TopMiddle,
    BottomMiddle,
    HorizontalLeft,
    HorizontalRight,
}

impl TableBorderChar {
    pub const fn as_char(self) -> char {
        match self {
            Self::TopLeft => '╭',
            Self::TopRight => '╮',
            Self::BottomLeft => '╰',
            Self::BottomRight => '╯',
            Self::Horizontal => '─',
            Self::Vertical => '│',
            Self::Middle => '┼',
            Self::TopMiddle => '┬',
            Self::BottomMiddle => '┴',
            Self::HorizontalLeft => '├',
            Self::HorizontalRight => '┤',
        }
    }
}

pub struct TableColumn {
    content: String,
    width: usize,
    align: TableAlignment,
}

impl TableColumn {
    pub fn new(content: impl Into<String>, width: usize) -> Self {
        Self {
            content: content.into(),
            width,
            align: TableAlignment::Left,
        }
    }

    pub fn with_alignment(mut self, align: TableAlignment) -> Self {
        self.align = align;
        self
    }
}

pub fn format_table_column(column: &TableColumn) -> String {
    let length = column.content.chars().count();

    if column.width <= length {
        return column.content.to_owned();
    }

    let padding = column.width - length;

    match column.align {
        TableAlignment::Left => format!("{}{}", column.content, " ".repeat(padding)),
        TableAlignment::Right => format!("{}{}", " ".repeat(padding), column.content),
        TableAlignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!(
                "{}{}{}",
                " ".repeat(left),
                column.content,
                " ".repeat(right)
            )
        }
    }
}

pub fn format_table_row(columns: &[TableColumn]) -> String {
    let row = columns
        .iter()
        .map(|c| format!(" {} ", format_table_column(&c)))
        .collect::<Vec<String>>()
        .join(&TableBorderChar::Vertical.as_char().to_string());
    format!(
        "{}{}{}",
        TableBorderChar::Vertical.as_char(),
        row,
        TableBorderChar::Vertical.as_char(),
    )
}

pub fn format_table_header(columns: &[TableColumn]) -> String {
    let top_border = columns
        .iter()
        .map(|c| {
            TableBorderChar::Horizontal
                .as_char()
                .to_string()
                .repeat(c.width + 2)
        })
        .collect::<Vec<String>>()
        .join(&TableBorderChar::TopMiddle.as_char().to_string());

    let header = format_table_row(&columns);

    let separator = columns
        .iter()
        .map(|c| {
            TableBorderChar::Horizontal
                .as_char()
                .to_string()
                .repeat(c.width + 2)
        })
        .collect::<Vec<String>>()
        .join(&TableBorderChar::Middle.as_char().to_string());

    format!(
        "{}{}{}\n{}\n{}{}{}",
        TableBorderChar::TopLeft.as_char(),
        top_border,
        TableBorderChar::TopRight.as_char(),
        header,
        TableBorderChar::HorizontalLeft.as_char(),
        separator,
        TableBorderChar::HorizontalRight.as_char()
    )
}

pub fn format_table_footer(columns: &[TableColumn]) -> String {
    let bottom_border = columns
        .iter()
        .map(|c| {
            TableBorderChar::Horizontal
                .as_char()
                .to_string()
                .repeat(c.width + 2)
        })
        .collect::<Vec<String>>()
        .join(&TableBorderChar::BottomMiddle.as_char().to_string());
    format!(
        "{}{}{}",
        TableBorderChar::BottomLeft.as_char(),
        bottom_border,
        TableBorderChar::BottomRight.as_char()
    )
}

pub struct TableColumnWidths {
    pub id: usize,
    pub name: usize,
    pub symbol: usize,
    pub public_key: usize,
    pub collateral_balance: usize,
    pub usd_balance: usize,
    pub allocation: usize,
    pub token_balance: usize,
    pub parameter: usize,
    pub entry_mcap: usize,
}
pub const TABLE_COLUMN_WIDTHS: TableColumnWidths = TableColumnWidths {
    id: 5,
    name: 12,
    symbol: 7,
    public_key: 42,
    collateral_balance: 24,
    usd_balance: 12,
    allocation: 10,
    token_balance: 20,
    parameter: 20,
    entry_mcap: 10,
};
