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

// Blockchain
pub const ETH_CHAIN_ID: u64 = 1;
pub const BNB_CHAIN_ID: u64 = 56;
pub const RH_CHAIN_ID: u64 = 4663;

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

// Misc
pub const PRICE_URL: &str = "https://api.binance.com/api/v3/ticker/price";
