use chrono::Utc;
use clap::Parser;
use tracing::{error, info};

use binance_aum_fetch::aum::calculate_aum;
use binance_aum_fetch::binance_client::BinanceClient;
use binance_aum_fetch::config::{AppConfig, Cli, OutputFormat};
use binance_aum_fetch::error::AppResult;
use binance_aum_fetch::models::AumReport;
use binance_aum_fetch::output;
use binance_aum_fetch::pricing::BinancePriceProvider;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        error!(error = %err, "binance_aum_fetch failed");
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> AppResult<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let cli = Cli::parse();
    let config = AppConfig::from_cli(cli)?;

    let client = BinanceClient::new(
        config.api_key.clone(),
        config.api_secret.clone(),
        config.api_base_url.clone(),
        config.papi_base_url.clone(),
        config.timeout,
    )?;

    let price_provider = BinancePriceProvider::new(client.clone(), config.quote_currency.clone());

    info!("binance_aum_fetch started");
    if config.once {
        let report = fetch_and_compute(&client, &price_provider, &config).await?;
        render(&report, config.output_format)?;
        return Ok(());
    }

    loop {
        match fetch_and_compute(&client, &price_provider, &config).await {
            Ok(report) => {
                if let Err(render_err) = render(&report, config.output_format) {
                    error!(error = %render_err, "failed to render report");
                }
            }
            Err(err) => {
                error!(error = %err, "failed to fetch/compute report");
            }
        }

        tokio::time::sleep(config.interval).await;
    }
}

async fn fetch_and_compute(
    client: &BinanceClient,
    price_provider: &BinancePriceProvider,
    config: &AppConfig,
) -> AppResult<AumReport> {
    let data = client
        .fetch_aum_data(&config.um_positions, &config.spot_assets)
        .await?;
    let calculation = calculate_aum(&data, price_provider).await?;

    Ok(AumReport {
        timestamp: Utc::now(),
        data,
        calculation,
    })
}

fn render(report: &AumReport, format: OutputFormat) -> AppResult<()> {
    match format {
        OutputFormat::Table => {
            output::render_table(report);
            Ok(())
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(report)?);
            Ok(())
        }
    }
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "binance_aum_fetch=info".into()),
        )
        .try_init();
}
