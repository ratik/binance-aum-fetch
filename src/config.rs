use clap::{Parser, ValueEnum};
use std::time::Duration;

use crate::error::{AppError, AppResult};

const DEFAULT_UM_POSITIONS: &str = "BTCUSDT,ETHUSDT,SOLUSDT";
const DEFAULT_SPOT_ASSETS: &str = "USDT,BTC,ETH,SOL";

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Parser)]
#[command(name = "binance_aum_fetch")]
#[command(about = "Fetches Binance data and calculates/display AUM")]
pub struct Cli {
    #[arg(long, env = "BINANCE_API_KEY")]
    pub binance_api_key: Option<String>,

    #[arg(long, env = "BINANCE_API_SECRET")]
    pub binance_api_secret: Option<String>,

    #[arg(long, env = "BINANCE_UM_POSITIONS", default_value = DEFAULT_UM_POSITIONS)]
    pub binance_um_positions: String,

    #[arg(long, env = "BINANCE_SPOT_ASSETS", default_value = DEFAULT_SPOT_ASSETS)]
    pub binance_spot_assets: String,

    #[arg(long, env = "QUOTE_CURRENCY", default_value = "USD")]
    pub quote_currency: String,

    #[arg(long, env = "OUTPUT_FORMAT", value_enum, default_value_t = OutputFormat::Table)]
    pub output_format: OutputFormat,

    #[arg(long, default_value_t = true)]
    pub once: bool,

    #[arg(long, default_value_t = 30)]
    pub interval: u64,

    #[arg(long, default_value_t = 10)]
    pub timeout: u64,

    #[arg(
        long,
        env = "BINANCE_API_BASE_URL",
        default_value = "https://api.binance.com"
    )]
    pub binance_api_base_url: String,

    #[arg(
        long,
        env = "BINANCE_PAPI_BASE_URL",
        default_value = "https://papi.binance.com"
    )]
    pub binance_papi_base_url: String,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub api_secret: String,
    pub um_positions: Vec<String>,
    pub spot_assets: Vec<String>,
    pub quote_currency: String,
    pub output_format: OutputFormat,
    pub once: bool,
    pub interval: Duration,
    pub timeout: Duration,
    pub api_base_url: String,
    pub papi_base_url: String,
}

impl AppConfig {
    pub fn from_cli(cli: Cli) -> AppResult<Self> {
        let api_key = cli
            .binance_api_key
            .filter(|v| !v.trim().is_empty())
            .ok_or(AppError::MissingConfig("BINANCE_API_KEY"))?;
        let api_secret = cli
            .binance_api_secret
            .filter(|v| !v.trim().is_empty())
            .ok_or(AppError::MissingConfig("BINANCE_API_SECRET"))?;

        let um_positions = parse_csv_symbols(&cli.binance_um_positions, "BINANCE_UM_POSITIONS")?;
        let spot_assets = parse_csv_symbols(&cli.binance_spot_assets, "BINANCE_SPOT_ASSETS")?;

        Ok(Self {
            api_key,
            api_secret,
            um_positions,
            spot_assets,
            quote_currency: cli.quote_currency.trim().to_uppercase(),
            output_format: cli.output_format,
            once: cli.once,
            interval: Duration::from_secs(cli.interval),
            timeout: Duration::from_secs(cli.timeout),
            api_base_url: trim_base_url(&cli.binance_api_base_url),
            papi_base_url: trim_base_url(&cli.binance_papi_base_url),
        })
    }
}

fn parse_csv_symbols(raw: &str, field: &'static str) -> AppResult<Vec<String>> {
    let values: Vec<String> = raw
        .split(',')
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(|v| v.to_uppercase())
        .collect();

    if values.is_empty() {
        return Err(AppError::InvalidConfig {
            field,
            reason: "value list must not be empty".to_string(),
        });
    }

    Ok(values)
}

fn trim_base_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}
