use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::error::{AppError, AppResult};
use crate::models::{AumCalculation, BinanceData, SpotContribution};
use crate::pricing::PriceProvider;

pub async fn calculate_aum<P: PriceProvider + Sync>(
    data: &BinanceData,
    prices: &P,
) -> AppResult<AumCalculation> {
    let mut spot_total_btc = Decimal::ZERO;
    let mut contributions = Vec::with_capacity(data.spot_balances.len());

    for spot in &data.spot_balances {
        let asset_upper = spot.asset.to_uppercase();
        let (btc_to_asset_price, amount_btc) = if asset_upper == "WBTC" {
            (Decimal::ONE, spot.amount)
        } else {
            let btc_to_asset = prices.btc_to_asset(&asset_upper).await?;
            if btc_to_asset.is_zero() {
                return Err(AppError::MissingPrice(asset_upper));
            }
            (btc_to_asset, spot.amount / btc_to_asset)
        };

        spot_total_btc += amount_btc;
        contributions.push(SpotContribution {
            asset: spot.asset.clone(),
            amount: spot.amount,
            btc_to_asset_price,
            amount_btc,
        });
    }

    let btc_usd_price = prices.btc_to_usd().await?;
    if btc_usd_price.is_zero() {
        return Err(AppError::MissingPrice("BTC/USD".to_string()));
    }

    let pm_equity_btc = data.pm_account_actual_equity / btc_usd_price;
    let aum_btc = pm_equity_btc + spot_total_btc;

    if aum_btc < Decimal::ZERO {
        return Err(AppError::NegativeAum(aum_btc.to_string()));
    }

    let aum_wbtc_u8 = (aum_btc * Decimal::from(100_000_000i64))
        .trunc()
        .to_i128()
        .ok_or_else(|| AppError::InvalidConfig {
            field: "aum_wbtc_u8",
            reason: "failed to convert to i128".to_string(),
        })?;

    let aum_wbtc = Decimal::from_i128_with_scale(aum_wbtc_u8, 8);

    Ok(AumCalculation {
        aum_btc_18dp: aum_btc,
        aum_wbtc_u8,
        aum_wbtc,
        spot_total_btc,
        pm_equity_usd: data.pm_account_actual_equity,
        btc_usd_price,
        spot_contributions: contributions,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::models::{BinanceData, SpotBalance, UmPosition};
    use async_trait::async_trait;

    #[derive(Debug)]
    struct MockPriceProvider {
        btc_usd: Decimal,
        btc_to_asset: HashMap<String, Decimal>,
    }

    #[async_trait]
    impl PriceProvider for MockPriceProvider {
        async fn btc_to_usd(&self) -> AppResult<Decimal> {
            Ok(self.btc_usd)
        }

        async fn btc_to_asset(&self, asset: &str) -> AppResult<Decimal> {
            self.btc_to_asset
                .get(asset)
                .cloned()
                .ok_or_else(|| AppError::MissingPrice(asset.to_string()))
        }
    }

    fn d(v: i64) -> Decimal {
        Decimal::from(v)
    }

    #[tokio::test]
    async fn computes_aum_with_pm_equity_only() {
        let data = BinanceData {
            unimmr: Decimal::ZERO,
            positions: vec![],
            um_balance_usdt: Decimal::ZERO,
            spot_balances: vec![],
            pm_account_actual_equity: d(200_000),
            withdrawable_usdt: Decimal::ZERO,
        };

        let prices = MockPriceProvider {
            btc_usd: d(100_000),
            btc_to_asset: HashMap::new(),
        };

        let result = calculate_aum(&data, &prices)
            .await
            .expect("calc should work");
        assert_eq!(result.aum_btc_18dp, Decimal::from_i128_with_scale(2, 0));
        assert_eq!(result.aum_wbtc_u8, 200_000_000);
    }

    #[tokio::test]
    async fn computes_aum_with_spot_conversion() {
        let data = BinanceData {
            unimmr: Decimal::ZERO,
            positions: vec![UmPosition {
                symbol: "BTCUSDT".to_string(),
                amount: Decimal::ONE,
                pnl: Decimal::ZERO,
            }],
            um_balance_usdt: Decimal::ZERO,
            spot_balances: vec![SpotBalance {
                asset: "ETH".to_string(),
                amount: d(1),
            }],
            pm_account_actual_equity: Decimal::ZERO,
            withdrawable_usdt: Decimal::ZERO,
        };

        let mut map = HashMap::new();
        map.insert("ETH".to_string(), d(50));

        let prices = MockPriceProvider {
            btc_usd: d(100_000),
            btc_to_asset: map,
        };

        let result = calculate_aum(&data, &prices)
            .await
            .expect("calc should work");
        assert_eq!(result.aum_btc_18dp, Decimal::from_i128_with_scale(2, 2));
        assert_eq!(result.aum_wbtc_u8, 2_000_000);
    }

    #[tokio::test]
    async fn rejects_negative_aum() {
        let data = BinanceData {
            unimmr: Decimal::ZERO,
            positions: vec![],
            um_balance_usdt: Decimal::ZERO,
            spot_balances: vec![],
            pm_account_actual_equity: d(-1),
            withdrawable_usdt: Decimal::ZERO,
        };

        let prices = MockPriceProvider {
            btc_usd: d(100_000),
            btc_to_asset: HashMap::new(),
        };

        let err = calculate_aum(&data, &prices)
            .await
            .expect_err("negative aum must fail");
        assert!(matches!(err, AppError::NegativeAum(_)));
    }
}
