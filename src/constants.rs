use crate::common::TableColumnWidths;
use alloy::primitives::{Address, address};

// CLI
pub const VERSION: &str = match option_env!("CARGO_PKG_VERSION") {
    Some(version) => version,
    None => "unknown",
};
pub const AUTHOR: &str = match option_env!("CARGO_PKG_AUTHORS") {
    Some(author) => author,
    None => "unknown",
};
pub const DESCRIPTION: &str = match option_env!("CARGO_PKG_DESCRIPTION") {
    Some(desc) => desc,
    None => "unknown",
};

// Commands
pub const DEFAULT_KEYS_FILE_PATH: &str = "keys.csv";
pub const ETH_RPC_ENV_KEY: &str = "ETH_RPC_URL";
pub const RH_RPC_ENV_KEY: &str = "RH_RPC_URL";
pub const BNB_RPC_ENV_KEY: &str = "BNB_RPC_URL";
pub const BASE_RPC_ENV_KEY: &str = "BASE_RPC_URL";
pub const INK_RPC_ENV_KEY: &str = "INK_RPC_URL";
pub const HYPER_RPC_ENV_KEY: &str = "HYPER_RPC_URL";
pub const ARC_RPC_ENV_KEY: &str = "ARC_RPC_URL";
pub const POLYGON_RPC_ENV_KEY: &str = "POLYGON_RPC_URL";

// Blockchain
pub const ETH_CHAIN_ID: u64 = 1;
pub const BNB_CHAIN_ID: u64 = 56;
pub const RH_CHAIN_ID: u64 = 4663;
pub const BASE_CHAIN_ID: u64 = 8453;
pub const INK_CHAIN_ID: u64 = 57073;
pub const HYPER_CHAIN_ID: u64 = 999;
pub const ARC_CHAIN_ID: u64 = 5042;
pub const POLYGON_CHAIN_ID: u64 = 137;

pub const MULTICALL3_ADDRESS: Address = address!("ca11bde05977b3631167028862be2a173976ca11");
pub const PERMIT2_ADDRESS: Address = address!("000000000022d473030f116ddee9f6b43ac78ba3");

pub const ETH_UNISWAP_UNIVERSAL_ROUTER_ADDRESS: Address =
    address!("66a9893cc07d91d95644aedd05d03f95e1dba8af");
pub const BNB_UNISWAP_UNIVERSAL_ROUTER_ADDRESS: Address =
    address!("1906c1d672b88cd1b9ac7593301ca990f94eae07");
pub const RH_UNISWAP_UNIVERSAL_ROUTER_ADDRESS: Address =
    address!("8876789976decbfcbbbe364623c63652db8c0904");

pub const BNB_PANCAKESWAP_V3_ROUTER_ADDRESS: Address =
    address!("1b81d678ffb9c0263b24a97847620c99d213eb14");

pub const TX_REQUIRED_CONFIRMATIONS: u64 = 2;

// Misc
pub const DEFILLAMA_PRICE_URL: &str = "https://coins.llama.fi/prices/current";
pub const DEXSCREENER_PRICE_URL: &str = "https://api.dexscreener.com/latest/dex/tokens";
pub const TABLE_COLUMN_WIDTHS: TableColumnWidths = TableColumnWidths {
    id: 5,
    name: 12,
    public_key: 42,
    collateral_balance: 24,
    usd_balance: 12,
    token_balance: 20,
    // symbol: 7,
    // allocation: 10,
    // parameter: 20,
    // entry_mcap: 10,
};
