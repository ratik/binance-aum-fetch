use crate::models::AumReport;

pub fn render_table(report: &AumReport) {
    println!("timestamp: {}", report.timestamp.to_rfc3339());
    println!("aum_wbtc_u8: {}", report.calculation.aum_wbtc_u8);
    println!("aum_wbtc: {}", report.calculation.aum_wbtc.round_dp(8));
    println!("aum_btc: {}", report.calculation.aum_btc_18dp.round_dp(18));
    println!(
        "spot_total_btc: {}",
        report.calculation.spot_total_btc.round_dp(18)
    );
    println!(
        "pm_equity_usd: {}",
        report.calculation.pm_equity_usd.round_dp(8)
    );
    println!(
        "btc_usd_price: {}",
        report.calculation.btc_usd_price.round_dp(8)
    );

    println!("spot_contributions:");
    for spot in &report.calculation.spot_contributions {
        println!(
            "  - {} amount={} btc_to_asset={} amount_btc={}",
            spot.asset,
            spot.amount.round_dp(18),
            spot.btc_to_asset_price.round_dp(18),
            spot.amount_btc.round_dp(18),
        );
    }

    println!("diagnostics:");
    println!("  - unimmr={}", report.data.unimmr.round_dp(8));
    println!(
        "  - um_balance_usdt={}",
        report.data.um_balance_usdt.round_dp(8)
    );
    println!(
        "  - withdrawable_usdt={}",
        report.data.withdrawable_usdt.round_dp(8)
    );
    println!("  - positions:");
    for p in &report.data.positions {
        println!(
            "    * {} amount={} pnl={}",
            p.symbol,
            p.amount.round_dp(18),
            p.pnl.round_dp(18)
        );
    }
}
