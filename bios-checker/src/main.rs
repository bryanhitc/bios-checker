use std::str::FromStr;

use bios_checker::check_bios_version;
use tokio::time::Instant;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_level = std::env::var("LOG_LEVEL")
        .ok()
        .and_then(|level| tracing::Level::from_str(&level).ok())
        .unwrap_or(tracing::Level::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            log_level,
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let start = Instant::now();
    let response = check_bios_version(String::from("1")).await?;
    tracing::info!(
        "Successfully checked bios version in {:?}: {:?}",
        start.elapsed(),
        response,
    );
    Ok(())
}
