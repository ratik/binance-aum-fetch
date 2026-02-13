use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue};
use rust_decimal::Decimal;
use serde::Deserialize;
use sha2::Sha256;
use url::form_urlencoded;

use crate::error::{AppError, AppResult};
use crate::models::{
    BinanceData, PmAccountBalanceApi, PmAccountInfoApi, SpotAccountInfoApi, SpotBalance,
    UmPosition, UmPositionApi,
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct BinanceClient {
    http: reqwest::Client,
    api_secret: String,
    api_base_url: String,
    papi_base_url: String,
}

#[derive(Debug, Deserialize)]
struct BinanceErrorBody {
    code: i64,
    msg: String,
}

impl BinanceClient {
    pub fn new(
        api_key: String,
        api_secret: String,
        api_base_url: String,
        papi_base_url: String,
        timeout: std::time::Duration,
    ) -> AppResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-MBX-APIKEY",
            HeaderValue::from_str(&api_key).map_err(|e| AppError::InvalidConfig {
                field: "BINANCE_API_KEY",
                reason: e.to_string(),
            })?,
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(timeout)
            .build()?;

        Ok(Self {
            http,
            api_secret,
            api_base_url,
            papi_base_url,
        })
    }

    pub async fn fetch_aum_data(
        &self,
        um_positions_list: &[String],
        spot_assets_list: &[String],
    ) -> AppResult<BinanceData> {
        let (um_positions, pm_account_info, pm_account_balances, spot_account_info) = tokio::try_join!(
            self.get_um_positions(),
            self.get_pm_account_info(),
            self.get_pm_account_balances(),
            self.get_spot_account_info(),
        )?;

        let positions = filter_positions(&um_positions, um_positions_list)?;
        let spot_balances = filter_spot_balances(&spot_account_info, spot_assets_list)?;

        let um_balance_usdt = pm_account_balances
            .iter()
            .find(|b| b.asset == "USDT")
            .map(|b| parse_decimal("umWalletBalance", &b.um_wallet_balance))
            .transpose()?
            .unwrap_or(Decimal::ZERO);

        Ok(BinanceData {
            unimmr: parse_decimal("uniMMR", &pm_account_info.uni_mmr)?,
            positions,
            um_balance_usdt,
            spot_balances,
            pm_account_actual_equity: parse_decimal(
                "actualEquity",
                &pm_account_info.actual_equity,
            )?,
            withdrawable_usdt: parse_decimal(
                "virtualMaxWithdrawAmount",
                &pm_account_info.virtual_max_withdraw_amount,
            )?,
        })
    }

    pub async fn ticker_price(&self, symbol: &str) -> AppResult<Decimal> {
        let endpoint = "/api/v3/ticker/price";
        let params = [("symbol", symbol.to_string())];
        let ticker: crate::models::PriceTickerApi = self
            .get_public(&self.api_base_url, endpoint, &params)
            .await?;
        parse_decimal("price", &ticker.price)
    }

    async fn get_um_positions(&self) -> AppResult<Vec<UmPositionApi>> {
        self.get_signed(&self.papi_base_url, "/papi/v1/um/positionRisk", &[])
            .await
    }

    async fn get_pm_account_info(&self) -> AppResult<PmAccountInfoApi> {
        self.get_signed(&self.papi_base_url, "/papi/v1/account", &[])
            .await
    }

    async fn get_pm_account_balances(&self) -> AppResult<Vec<PmAccountBalanceApi>> {
        self.get_signed(&self.papi_base_url, "/papi/v1/balance", &[])
            .await
    }

    async fn get_spot_account_info(&self) -> AppResult<SpotAccountInfoApi> {
        self.get_signed(&self.api_base_url, "/api/v3/account", &[])
            .await
    }

    async fn get_public<T: serde::de::DeserializeOwned>(
        &self,
        base_url: &str,
        endpoint: &str,
        params: &[(&str, String)],
    ) -> AppResult<T> {
        let url = format!("{}{}", base_url, endpoint);
        let query = build_query(params.iter().map(|(k, v)| (*k, v.as_str())));
        let request = if query.is_empty() {
            self.http.get(url)
        } else {
            self.http.get(format!("{url}?{query}"))
        };

        let response = request.send().await?;
        parse_response(response).await
    }

    async fn get_signed<T: serde::de::DeserializeOwned>(
        &self,
        base_url: &str,
        endpoint: &str,
        params: &[(&str, String)],
    ) -> AppResult<T> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let timestamp_string = timestamp.to_string();

        let mut pairs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        pairs.push(("timestamp", &timestamp_string));

        let mut query = build_query(pairs);
        let signature = sign_query(&query, &self.api_secret)?;
        if !query.is_empty() {
            query.push('&');
        }
        query.push_str("signature=");
        query.push_str(&signature);

        let url = format!("{}{}?{}", base_url, endpoint, query);
        let response = self.http.get(url).send().await?;
        parse_response(response).await
    }
}

fn build_query<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (k, v) in pairs {
        serializer.append_pair(k, v);
    }
    serializer.finish()
}

fn sign_query(query: &str, secret: &str) -> AppResult<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| AppError::Signature)?;
    mac.update(query.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

async fn parse_response<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> AppResult<T> {
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        if let Ok(err) = serde_json::from_str::<BinanceErrorBody>(&body) {
            return Err(AppError::BinanceApiMessage {
                code: err.code,
                msg: err.msg,
            });
        }
        return Err(AppError::BinanceApi {
            status: status.as_u16(),
            body,
        });
    }

    Ok(serde_json::from_str(&body)?)
}

fn parse_decimal(field: &'static str, value: &str) -> AppResult<Decimal> {
    Decimal::from_str_exact(value).map_err(|_| AppError::DecimalParse {
        field,
        value: value.to_string(),
    })
}

fn filter_positions(
    positions: &[UmPositionApi],
    required_symbols: &[String],
) -> AppResult<Vec<UmPosition>> {
    let mut filtered = Vec::new();
    for position in positions {
        if required_symbols.contains(&position.symbol) {
            filtered.push(UmPosition {
                symbol: position.symbol.clone(),
                amount: parse_decimal("positionAmt", &position.position_amt)?,
                pnl: parse_decimal("unrealizedProfit", &position.unrealized_profit)?,
            });
        }
    }
    Ok(filtered)
}

fn filter_spot_balances(
    account_info: &SpotAccountInfoApi,
    required_assets: &[String],
) -> AppResult<Vec<SpotBalance>> {
    let mut filtered = Vec::new();
    for balance in &account_info.balances {
        if required_assets.contains(&balance.asset) {
            let free = parse_decimal("free", &balance.free)?;
            let locked = parse_decimal("locked", &balance.locked)?;
            filtered.push(SpotBalance {
                asset: balance.asset.clone(),
                amount: free + locked,
            });
        }
    }
    Ok(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn query_signing_is_stable() {
        let signature = sign_query("timestamp=123", "secret").expect("signature should work");
        assert_eq!(
            signature,
            "49a8d551f916f1f7fd6956b49f3ea8c8e1f955490f8e19b5fb0bed82dbe6fd9b"
        );
    }

    #[test]
    fn filters_spot_and_sums_free_locked() {
        let payload: SpotAccountInfoApi = serde_json::from_str(include_str!(
            "../../aum_messenger/testutil/clients-mock-controller/mock_data/binance/spotAccountInfo.json"
        ))
        .expect("spot payload should decode");

        let out = filter_spot_balances(&payload, &["BTC".to_string(), "USDT".to_string()])
            .expect("filter should work");

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].asset, "BTC");
        assert_eq!(out[1].asset, "USDT");
    }
}
