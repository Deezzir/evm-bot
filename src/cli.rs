use crate::chain::{Chain, DexProtocol, LaunchpadProtocol, TradingProtocol};
use crate::constants::{AUTHOR, DEFAULT_KEYS_FILE_PATH, DESCRIPTION, VERSION};
use clap::{Parser, Subcommand, ValueEnum};
use figlet_rs::FIGlet;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate wallets and save them to a CSV file
    #[command[alias = "g"]]
    Generate {
        /// Path to the CSV file to save the generated wallets
        file_path: PathBuf,
        /// Generate the reserve wallet
        #[arg(long = "reserve", short = 'r', default_value_t)]
        reserve: bool,
        /// Starting index of the wallets to generate
        #[arg(long = "index", short = 'i')]
        index: Option<usize>,
        /// Number of wallets to generate
        #[arg(long = "count", short = 'c', default_value_t = 0)]
        count: usize,
        /// Path to the file with with secret keys to convert
        #[arg(long = "secrets_path", short = 's')]
        secrets_path: Option<PathBuf>,
    },
    /// Get the balance of the wallets
    #[command[alias = "b"]]
    Balance {
        /// Format of the balance output
        #[arg(long = "format", short = 'f', value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    /// Get the token balance of the wallets
    #[command[alias = "tb"]]
    TokenBalance {
        /// Token mint address
        mint: String,
        /// Format of the token balance output
        #[arg(long = "format", short = 'f', value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    /// Transfer from the specified wallet to the receiver
    #[command[alias = "tr"]]
    Transfer {
        /// The amount to transfer
        amount: String,
        /// The receiver public address
        receiver: String,
        /// The index of the sender wallet
        sender_index: usize,
    },
    /// Transfer token from the specified wallet to the receiver
    #[command[alias = "tt"]]
    TokenTransfer {
        /// The token mint address
        mint: String,
        /// The amount to transfer
        amount: String,
        /// The receiver public address
        receiver: String,
        /// The index of the sender wallet
        sender_index: usize,
    },
    /// Buy a token through a DEX or launchpad
    BuyTokenOnce {
        /// Native-token amount to spend
        amount: f64,
        /// Token contract address
        token: String,
        /// Index of the buyer wallet
        index: usize,
        /// Trading venue to use
        #[arg(long, value_enum)]
        venue: TradingVenueArg,
    },
    SellTokenOnce {
        /// Native-token amount to receive
        amount: f64,
        /// Token contract address
        token: String,
        /// Index of the seller wallet
        index: usize,
        /// Trading venue to use
        #[arg(long, value_enum)]
        venue: TradingVenueArg,
    },
    /// Fund the wallets with collateral using the provided wallet
    Fund {
        /// Amount of collateral to fund each wallet with
        amount: f64,
        /// Index of the wallet to use for funding
        sender_index: usize,
        /// Starting from the provided index
        #[arg(long = "from", short = 'f')]
        from: Option<usize>,
        /// Ending at the provided index (exclusive)
        #[arg(long = "to", short = 't')]
        to: Option<usize>,
        /// Specify the list of wallet indexes to fund (overrides `from` and `to`)
        #[arg(
            long = "indexes",
            short = 'i',
            value_delimiter = ',',
            conflicts_with_all = ["from", "to"]
        )]
        indexes: Option<Vec<usize>>,
        /// Fund randomly using <amount> argument as a mean value
        #[arg(long = "random", short = 'r')]
        random: Option<bool>,
    },
    /// Collect all the collateral from the wallets to the provided address
    Collect {
        /// Public address of the receiver
        receiver: String,
        /// Starting from the provided index
        #[arg(long = "from", short = 'f')]
        from: Option<usize>,
        /// Ending at the provided index (exclusive)
        #[arg(long = "to", short = 't')]
        to: Option<usize>,
        /// Specify the list of wallet indexes to collect from (overrides `from` and `to`)
        #[arg(
            long = "indexes",
            short = 'i',
            value_delimiter = ',',
            conflicts_with_all = ["from", "to"]
        )]
        indexes: Option<Vec<usize>>,
    },
    /// Collect all the tokens from the wallets to the provided address
    CollectTokens {
        /// Token mint address
        mint: String,
        /// Public address of the receiver
        receiver: String,
        /// Starting from the provided index
        #[arg(long = "from", short = 'f')]
        from: Option<usize>,
        /// Ending at the provided index (exclusive)
        #[arg(long = "to", short = 't')]
        to: Option<usize>,
        /// Specify the list of wallet indexes to collect from (overrides `from` and `to`)
        #[arg(
            long = "indexes",
            short = 'i',
            value_delimiter = ',',
            conflicts_with_all = ["from", "to"]
        )]
        indexes: Option<Vec<usize>>,
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
pub struct Cli {
    /// Blockchain to use
    #[arg(long = "chain", short = 'c', value_enum, default_value_t = Chain::Ethereum)]
    pub chain: Chain,

    /// Path to the CSV file with the wallets
    #[arg(long = "keys_path", short = 'k', default_value = DEFAULT_KEYS_FILE_PATH)]
    pub keys_path: PathBuf,

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
