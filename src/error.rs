use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("missing configuration value: {0}")]
    MissingConfig(&'static str),

    #[error("invalid configuration value for {field}: {reason}")]
    InvalidConfig { field: &'static str, reason: String },

    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to parse decimal from `{field}` value `{value}`")]
    DecimalParse { field: &'static str, value: String },

    #[error("failed to decode json payload: {0}")]
    Json(#[from] serde_json::Error),

    #[error("binance api returned error status {status}: {body}")]
    BinanceApi { status: u16, body: String },

    #[error("binance api error {code}: {msg}")]
    BinanceApiMessage { code: i64, msg: String },

    #[error("signature generation failed")]
    Signature,

    #[error("time error: {0}")]
    Time(#[from] std::time::SystemTimeError),

    #[error("price unavailable for asset `{0}`")]
    MissingPrice(String),

    #[error("negative aum computed: {0}")]
    NegativeAum(String),
}

pub type AppResult<T> = Result<T, AppError>;
