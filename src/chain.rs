use crate::constants;
use alloy::primitives::Address;
use anyhow::{Result, bail};
use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DexProtocol {
    #[value(name = "uniswapv3")]
    UniswapV3,
    #[value(name = "pancakeswapv3")]
    PancakeSwapV3,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chain {
    #[value(name = "ethereum", alias = "eth")]
    Ethereum,
    #[value(name = "robinhood", alias = "rh")]
    Robinhood,
    #[value(name = "bnb", alias = "bnb")]
    Bnb,
    #[value(name = "base")]
    Base,
    #[value(name = "ink")]
    Ink,
    #[value(name = "hyper", alias = "hyperliquid")]
    Hyper,
    #[value(name = "arc")]
    Arc,
    #[value(name = "polygon", alias = "pol")]
    Polygon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollateralToken {
    Eth,
    Bnb,
    Hype,
    Usdc,
    Pol,
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

impl Chain {
    pub const fn config(self) -> ChainConfig {
        let (chain_id, rpc_env_key) = match self {
            Self::Ethereum => (constants::ETH_CHAIN_ID, constants::ETH_RPC_ENV_KEY),
            Self::Bnb => (constants::BNB_CHAIN_ID, constants::BNB_RPC_ENV_KEY),
            Self::Robinhood => (constants::RH_CHAIN_ID, constants::RH_RPC_ENV_KEY),
            Self::Base => (constants::BASE_CHAIN_ID, constants::BASE_RPC_ENV_KEY),
            Self::Ink => (constants::INK_CHAIN_ID, constants::INK_RPC_ENV_KEY),
            Self::Hyper => (constants::HYPER_CHAIN_ID, constants::HYPER_RPC_ENV_KEY),
            Self::Arc => (constants::ARC_CHAIN_ID, constants::ARC_RPC_ENV_KEY),
            Self::Polygon => (constants::POLYGON_CHAIN_ID, constants::POLYGON_RPC_ENV_KEY),
        };

        ChainConfig {
            chain: self,
            chain_id,
            rpc_env_key,
            multicall3: constants::MULTICALL3_ADDRESS,
        }
    }

    pub fn dex(self, protocol: DexProtocol) -> Result<DexDeployment> {
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
                bail!("{} is not supported on {:?}", protocol.name(), self);
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

    pub fn launchpad(self, protocol: LaunchpadProtocol) -> Result<LaunchpadDeployment> {
        match (self, protocol) {
            (Self::Bnb, LaunchpadProtocol::FourMeme)
            | (Self::Robinhood, LaunchpadProtocol::PonsFamily) => Ok(LaunchpadDeployment {
                chain: self,
                protocol,
            }),
            _ => bail!("{} is not supported on {:?}", protocol.name(), self),
        }
    }

    pub fn trading(self, protocol: TradingProtocol) -> Result<TradingDeployment> {
        match protocol {
            TradingProtocol::Dex(protocol) => self.dex(protocol).map(TradingDeployment::Dex),
            TradingProtocol::Launchpad(protocol) => {
                self.launchpad(protocol).map(TradingDeployment::Launchpad)
            }
        }
    }

    pub fn collateral(self) -> Result<CollateralToken> {
        match self {
            Self::Ethereum | Self::Robinhood | Self::Base | Self::Ink => Ok(CollateralToken::Eth),
            Self::Bnb => Ok(CollateralToken::Bnb),
            Self::Hyper => Ok(CollateralToken::Hype),
            Self::Arc => Ok(CollateralToken::Usdc),
            Self::Polygon => Ok(CollateralToken::Pol),
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

impl CollateralToken {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bnb => "BNB",
            Self::Eth => "ETH",
            Self::Hype => "HYPE",
            Self::Usdc => "USDC",
            Self::Pol => "POL",
        }
    }
}
