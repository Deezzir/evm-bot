use std::{error::Error, future::Future};

use alloy::primitives::{Address, B256, U256};

use crate::chain::{DexDeployment, LaunchpadDeployment};

#[derive(Debug, Clone)]
pub struct SwapRequest {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub minimum_amount_out: U256,
    pub recipient: Address,
}

#[derive(Debug, Clone)]
pub struct SwapQuote {
    pub amount_out: U256,
}

#[derive(Debug, Clone)]
pub struct CreateTokenRequest {
    pub name: String,
    pub symbol: String,
    pub metadata_uri: String,
}

#[derive(Debug, Clone)]
pub struct CreatedToken {
    pub token: Address,
    pub transaction_hash: B256,
}

pub trait SwapVenue {
    type Error: Error + Send + Sync + 'static;

    fn deployment(&self) -> DexDeployment;

    fn quote(
        &self,
        request: &SwapRequest,
    ) -> impl Future<Output = Result<SwapQuote, Self::Error>> + Send;

    fn swap(&self, request: SwapRequest) -> impl Future<Output = Result<B256, Self::Error>> + Send;
}

pub trait LaunchpadVenue {
    type Error: Error + Send + Sync + 'static;

    fn deployment(&self) -> LaunchpadDeployment;

    fn create_token(
        &self,
        request: CreateTokenRequest,
    ) -> impl Future<Output = Result<CreatedToken, Self::Error>> + Send;

    fn buy(&self, request: SwapRequest) -> impl Future<Output = Result<B256, Self::Error>> + Send;

    fn sell(&self, request: SwapRequest) -> impl Future<Output = Result<B256, Self::Error>> + Send;
}
