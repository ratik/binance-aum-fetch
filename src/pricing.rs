use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::binance_client::BinanceClient;
use crate::error::{AppError, AppResult};

#[async_trait]
pub trait PriceProvider {
    async fn btc_to_usd(&self) -> AppResult<Decimal>;
    async fn btc_to_asset(&self, asset: &str) -> AppResult<Decimal>;
}

#[derive(Debug, Clone)]
pub struct BinancePriceProvider {
    client: BinanceClient,
    quote_currency: String,
}

impl BinancePriceProvider {
    pub fn new(client: BinanceClient, quote_currency: String) -> Self {
        Self {
            client,
            quote_currency,
        }
    }

    async fn ticker_or_none(&self, symbol: &str) -> AppResult<Option<Decimal>> {
        match self.client.ticker_price(symbol).await {
            Ok(price) => Ok(Some(price)),
            Err(AppError::BinanceApiMessage { code, .. }) if code == -1121 => Ok(None),
            Err(err) => Err(err),
        }
    }
}

#[async_trait]
impl PriceProvider for BinancePriceProvider {
    async fn btc_to_usd(&self) -> AppResult<Decimal> {
        let symbol = format!("BTC{}", self.quote_currency);
        self.client.ticker_price(&symbol).await
    }

    async fn btc_to_asset(&self, asset: &str) -> AppResult<Decimal> {
        let asset = asset.to_uppercase();
        if asset == "BTC" {
            return Ok(Decimal::ONE);
        }

        if asset == self.quote_currency {
            return self.btc_to_usd().await;
        }

        let direct_symbol = format!("BTC{}", asset);
        if let Some(price) = self.ticker_or_none(&direct_symbol).await? {
            return Ok(price);
        }

        let inverse_symbol = format!("{}BTC", asset);
        if let Some(price) = self.ticker_or_none(&inverse_symbol).await? {
            if price.is_zero() {
                return Err(AppError::MissingPrice(asset));
            }
            return Ok(Decimal::ONE / price);
        }

        Err(AppError::MissingPrice(asset))
    }
}
