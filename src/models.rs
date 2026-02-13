use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UmPositionApi {
    pub symbol: String,
    pub position_amt: String,
    #[serde(rename = "unrealizedProfit", alias = "unRealizedProfit")]
    pub unrealized_profit: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PmAccountInfoApi {
    #[serde(rename = "uniMMR", alias = "uniMmr")]
    pub uni_mmr: String,
    pub actual_equity: String,
    pub virtual_max_withdraw_amount: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PmAccountBalanceApi {
    pub asset: String,
    pub um_wallet_balance: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotAccountInfoApi {
    pub balances: Vec<SpotBalanceApi>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotBalanceApi {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceTickerApi {
    pub price: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UmPosition {
    pub symbol: String,
    pub amount: Decimal,
    pub pnl: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpotBalance {
    pub asset: String,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct BinanceData {
    pub unimmr: Decimal,
    pub positions: Vec<UmPosition>,
    pub um_balance_usdt: Decimal,
    pub spot_balances: Vec<SpotBalance>,
    pub pm_account_actual_equity: Decimal,
    pub withdrawable_usdt: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpotContribution {
    pub asset: String,
    pub amount: Decimal,
    pub btc_to_asset_price: Decimal,
    pub amount_btc: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct AumCalculation {
    pub aum_btc_18dp: Decimal,
    pub aum_wbtc_u8: i128,
    pub aum_wbtc: Decimal,
    pub spot_total_btc: Decimal,
    pub pm_equity_usd: Decimal,
    pub btc_usd_price: Decimal,
    pub spot_contributions: Vec<SpotContribution>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AumReport {
    pub timestamp: DateTime<Utc>,
    pub data: BinanceData,
    pub calculation: AumCalculation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_binance_mock_payloads() {
        let positions: Vec<UmPositionApi> =
            serde_json::from_str(include_str!("../../aum_messenger/testutil/clients-mock-controller/mock_data/binance/umPositions.json"))
                .expect("positions json should decode");
        assert!(!positions.is_empty());

        let account: PmAccountInfoApi =
            serde_json::from_str(include_str!("../../aum_messenger/testutil/clients-mock-controller/mock_data/binance/pmAccountInfo.json"))
                .expect("account json should decode");
        assert_eq!(account.uni_mmr, "76.77211871");

        let balances: Vec<PmAccountBalanceApi> =
            serde_json::from_str(include_str!("../../aum_messenger/testutil/clients-mock-controller/mock_data/binance/pmAccountBalance.json"))
                .expect("pm balances json should decode");
        assert!(!balances.is_empty());

        let spot: SpotAccountInfoApi =
            serde_json::from_str(include_str!("../../aum_messenger/testutil/clients-mock-controller/mock_data/binance/spotAccountInfo.json"))
                .expect("spot account json should decode");
        assert!(!spot.balances.is_empty());
    }
}
