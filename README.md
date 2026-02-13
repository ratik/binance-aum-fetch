# Binance AUM fetcher

Rust crate for fetching Binance account data and computing AUM

It can be used as:
- a CLI (`src/main.rs`)
- a reusable library (`src/lib.rs`)

## Add As Library

Example usage:

```rust
use binance_aum_fetch::aum::calculate_aum;
use binance_aum_fetch::binance_client::BinanceClient;
use binance_aum_fetch::pricing::BinancePriceProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BinanceClient::new(
        std::env::var("BINANCE_API_KEY")?,
        std::env::var("BINANCE_API_SECRET")?,
        std::env::var("BINANCE_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.binance.com".to_string()),
        std::env::var("BINANCE_PAPI_BASE_URL")
            .unwrap_or_else(|_| "https://papi.binance.com".to_string()),
        std::time::Duration::from_secs(10),
    )?;

    let data = client
        .fetch_aum_data(
            &["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            &["USDT".to_string(), "BTC".to_string(), "ETH".to_string()],
        )
        .await?;

    let prices = BinancePriceProvider::new(client.clone(), "USD".to_string());
    let calc = calculate_aum(&data, &prices).await?;

    println!("aum_wbtc_u8: {}", calc.aum_wbtc_u8);
    Ok(())
}
```

## Run

```bash
git clone https://github.com/ratik/binance-aum-fetch
cd binance-aum-fetch
cargo run -- --once
```

Environment variables (or matching CLI flags):

```bash
BINANCE_API_KEY=...
BINANCE_API_SECRET=...
# defaults:
# BINANCE_API_BASE_URL=https://api.binance.com
# BINANCE_PAPI_BASE_URL=https://papi.binance.com
```

## JSON output

```bash
cargo run -- --output-format json --once
```

## License

This project is licensed under the NON-AI-MIT license.
See `LICENSE`.
