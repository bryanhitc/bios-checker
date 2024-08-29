use std::str::FromStr;

use anyhow::Result;
use lambda_runtime::tower::BoxError;
use lambda_runtime::{service_fn, LambdaEvent};
use serde_json::Value;

use bios_checker::{check_bios_version, Response};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
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

    lambda_runtime::run(service_fn(lambda_handler)).await?;
    Ok(())
}

async fn lambda_handler(request: LambdaEvent<Value>) -> Result<Response, BoxError> {
    check_bios_version(request.context.request_id)
        .await
        .map_err(BoxError::from)
}
