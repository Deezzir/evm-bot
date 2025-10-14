use std::{error::Error, fmt};

use alloy::primitives::Address;
use clap::ValueEnum;

use crate::constants;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chain {
    #[value(name = "ethereum", alias = "eth")]
    Ethereum,
    #[value(name = "robinhood", alias = "rh")]
    Robinhood,
    #[value(name = "bnb", alias = "bnb")]
    Bnb,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DexProtocol {
    #[value(name = "uniswapv3")]
    UniswapV3,
    #[value(name = "pancakeswapv3")]
    PancakeSwapV3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchpadProtocol {
    FourMeme,
    PonsFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradingProtocol {
    Dex(DexProtocol),
    Launchpad(LaunchpadProtocol),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouterKind {
    UniversalRouter,
    V3Router,
}

#[derive(Debug, Clone, Copy)]
pub struct ChainConfig {
    pub chain: Chain,
    pub chain_id: u64,
    pub rpc_env_key: &'static str,
    pub multicall3: Address,
}

#[derive(Debug, Clone, Copy)]
pub struct DexDeployment {
    pub chain: Chain,
    pub protocol: DexProtocol,
    pub router_kind: RouterKind,
    pub router: Address,
    pub permit2: Option<Address>,
}

#[derive(Debug, Clone, Copy)]
pub struct LaunchpadDeployment {
    pub chain: Chain,
    pub protocol: LaunchpadProtocol,
}

#[derive(Debug, Clone, Copy)]
pub enum TradingDeployment {
    Dex(DexDeployment),
    Launchpad(LaunchpadDeployment),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedDeployment {
    pub chain: Chain,
    pub protocol: &'static str,
}

impl fmt::Display for UnsupportedDeployment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} is not supported on {:?}",
            self.protocol, self.chain
        )
    }
}

impl Error for UnsupportedDeployment {}

impl Chain {
    pub const fn config(self) -> ChainConfig {
        let (chain_id, rpc_env_key) = match self {
            Self::Ethereum => (constants::ETH_CHAIN_ID, constants::ETH_RPC_ENV_KEY),
            Self::Bnb => (constants::BNB_CHAIN_ID, constants::BNB_RPC_ENV_KEY),
            Self::Robinhood => (constants::RH_CHAIN_ID, constants::RH_RPC_ENV_KEY),
        };

        ChainConfig {
            chain: self,
            chain_id,
            rpc_env_key,
            multicall3: constants::MULTICALL3_ADDRESS,
        }
    }

    pub fn dex(self, protocol: DexProtocol) -> Result<DexDeployment, UnsupportedDeployment> {
        let (router_kind, router, permit2) = match (self, protocol) {
            (Self::Ethereum, DexProtocol::UniswapV3) => (
                RouterKind::UniversalRouter,
                constants::ETH_UNISWAP_UNIVERSAL_ROUTER_ADDRESS,
                Some(constants::PERMIT2_ADDRESS),
            ),
            (Self::Bnb, DexProtocol::UniswapV3) => (
                RouterKind::UniversalRouter,
                constants::BNB_UNISWAP_UNIVERSAL_ROUTER_ADDRESS,
                Some(constants::PERMIT2_ADDRESS),
            ),
            (Self::Robinhood, DexProtocol::UniswapV3) => (
                RouterKind::UniversalRouter,
                constants::RH_UNISWAP_UNIVERSAL_ROUTER_ADDRESS,
                Some(constants::PERMIT2_ADDRESS),
            ),
            (Self::Bnb, DexProtocol::PancakeSwapV3) => (
                RouterKind::V3Router,
                constants::BNB_PANCAKESWAP_V3_ROUTER_ADDRESS,
                None,
            ),
            _ => {
                return Err(UnsupportedDeployment {
                    chain: self,
                    protocol: protocol.name(),
                });
            }
        };

        Ok(DexDeployment {
            chain: self,
            protocol,
            router_kind,
            router,
            permit2,
        })
    }

    pub fn launchpad(
        self,
        protocol: LaunchpadProtocol,
    ) -> Result<LaunchpadDeployment, UnsupportedDeployment> {
        match (self, protocol) {
            (Self::Bnb, LaunchpadProtocol::FourMeme)
            | (Self::Robinhood, LaunchpadProtocol::PonsFamily) => Ok(LaunchpadDeployment {
                chain: self,
                protocol,
            }),
            _ => Err(UnsupportedDeployment {
                chain: self,
                protocol: protocol.name(),
            }),
        }
    }

    pub fn trading(
        self,
        protocol: TradingProtocol,
    ) -> Result<TradingDeployment, UnsupportedDeployment> {
        match protocol {
            TradingProtocol::Dex(protocol) => self.dex(protocol).map(TradingDeployment::Dex),
            TradingProtocol::Launchpad(protocol) => {
                self.launchpad(protocol).map(TradingDeployment::Launchpad)
            }
        }
    }
}

impl DexProtocol {
    pub const fn name(self) -> &'static str {
        match self {
            Self::UniswapV3 => "Uniswap V3",
            Self::PancakeSwapV3 => "PancakeSwap V3",
        }
    }
}

impl LaunchpadProtocol {
    pub const fn name(self) -> &'static str {
        match self {
            Self::FourMeme => "Four.meme",
            Self::PonsFamily => "Pons Family",
        }
    }
}
