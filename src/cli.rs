use crate::chain::{Chain, DexProtocol, LaunchpadProtocol, TradingProtocol};
use crate::constants::{AUTHOR, DEFAULT_KEYS_FILE, DESCRIPTION, VERSION};
use clap::{Parser, Subcommand, ValueEnum};
use figlet_rs::FIGlet;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get the balance of the wallets
    #[command[alias = "b"]]
    Balance {
        /// Format of the balance output
        #[arg(long = "format", short = 'f', value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    /// Generate wallets and save them to a CSV file
    #[command[alias = "g"]]
    Generate {
        file_path: String,
        /// Generate the reserve wallet
        #[arg(long = "reserve", short = 'r', default_value_t)]
        reserve: bool,
        /// Starting index of the wallets to generate
        #[arg(long = "index", short = 'i', default_value_t = 0)]
        index: usize,
        /// Number of wallets to generate
        #[arg(long = "count", short = 'c', default_value_t = 1)]
        count: usize,
        /// path to the file with with secret keys to convert
        #[arg(long = "secrets", short = 's')]
        secrets_path: Option<String>,
    },
    /// Get the token balance of the wallets
    #[command[alias = "tb"]]
    TokenBalance {
        mint: String,
        /// Format of the token balance output
        #[arg(long = "format", short = 'f', value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    /// Transfer from the specified wallet to the receiver
    #[command[alias = "tr"]]
    Transfer {
        /// The amount to transfer
        amount: f64,
        /// The index of the sender wallet
        #[arg(long = "index", short = 'i', default_value_t = 0)]
        index: usize,
        /// The receiver public address
        receiver: String,
    },
    /// Transfer token from the specified wallet to the receiver
    #[command[alias = "tt"]]
    TokenTransfer {
        /// The token mint address
        mint: String,
        /// The amount to transfer
        amount: f64,
        /// The index of the sender wallet
        #[arg(long = "index", short = 'i', default_value_t = 0)]
        index: usize,
        /// The receiver public address
        receiver: String,
    },
    /// Buy a token through a DEX or launchpad
    Buy {
        /// Token contract address
        token: String,
        /// Native-token amount to spend
        amount: f64,
        /// Trading venue to use
        #[arg(long, value_enum)]
        venue: TradingVenueArg,
        /// Index of the buyer wallet
        #[arg(long = "index", short = 'i', default_value_t = 0)]
        index: usize,
    },
    /// Create a token on a launchpad
    CreateToken {
        name: String,
        symbol: String,
        metadata_uri: String,
        /// Launchpad to use
        #[arg(long, value_enum)]
        launchpad: LaunchpadArg,
        /// Index of the creator wallet
        #[arg(long = "index", short = 'i', default_value_t = 0)]
        index: usize,
    },
    /// Watch a launchpad and buy matching token launches
    Snipe {
        /// Launchpad to watch
        #[arg(long, value_enum)]
        launchpad: LaunchpadArg,
    },
}

#[derive(Parser, Debug)]
#[command(
    name = "bot",
    about = DESCRIPTION,
    version = VERSION,
    author = AUTHOR,
    before_help = get_banner()
)]
pub struct CLI {
    /// Blockchain to use
    #[arg(long = "chain", short = 'c', value_enum, default_value_t = Chain::Ethereum)]
    pub chain: Chain,

    /// Path to the CSV file with the wallets
    #[arg(long = "keys", short = 'k', default_value_t = DEFAULT_KEYS_FILE.to_string())]
    pub keys: String,

    /// Disable colored output
    #[arg(long = "no-colors", default_value_t)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Commands,
}

fn get_banner() -> String {
    FIGlet::slant()
        .unwrap()
        .convert("EVM Bot")
        .unwrap()
        .to_string()
        .trim_end()
        .to_string()
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    #[value(name = "json")]
    Json,
    #[value(name = "csv")]
    Csv,
    #[value(name = "table")]
    Table,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum TradingVenueArg {
    #[value(name = "uniswap-v3")]
    UniswapV3,
    #[value(name = "pancakeswap-v3")]
    PancakeSwapV3,
    #[value(name = "four-meme")]
    FourMeme,
    #[value(name = "pons-family")]
    PonsFamily,
}

impl From<TradingVenueArg> for TradingProtocol {
    fn from(value: TradingVenueArg) -> Self {
        match value {
            TradingVenueArg::UniswapV3 => Self::Dex(DexProtocol::UniswapV3),
            TradingVenueArg::PancakeSwapV3 => Self::Dex(DexProtocol::PancakeSwapV3),
            TradingVenueArg::FourMeme => Self::Launchpad(LaunchpadProtocol::FourMeme),
            TradingVenueArg::PonsFamily => Self::Launchpad(LaunchpadProtocol::PonsFamily),
        }
    }
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum LaunchpadArg {
    #[value(name = "four-meme")]
    FourMeme,
    #[value(name = "pons-family")]
    PonsFamily,
}

impl From<LaunchpadArg> for LaunchpadProtocol {
    fn from(value: LaunchpadArg) -> Self {
        match value {
            LaunchpadArg::FourMeme => Self::FourMeme,
            LaunchpadArg::PonsFamily => Self::PonsFamily,
        }
    }
}
