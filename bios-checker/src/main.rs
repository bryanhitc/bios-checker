use std::str::FromStr;
use std::time::Instant;

use anyhow::Result;
use lambda_runtime::tower::BoxError;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, error, info, warn};
use tracing_subscriber::prelude::*;

use bios_checker::bios;

use lambda_notifiers::notifiers::discord::DiscordNotifier;
use lambda_notifiers::notifiers::email::{EmailContact, EmailMessage, EmailNotifier};
use lambda_notifiers::Notifier;

#[derive(Debug, Serialize)]
struct Response {
    req_id: String,
    expected_version: u32,
    latest_version: u32,
    notification_sent: bool,
}

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

    lambda_runtime::run(service_fn(my_handler)).await?;

    Ok(())
}

async fn my_handler(request: LambdaEvent<Value>) -> Result<Response, BoxError> {
    let start = Instant::now();
    info!("Handling request: {:?}", request);

    let expected_version = std::env::var("LATEST_VER")?;
    let expected_version = expected_version
        .parse::<u32>()
        .expect("The expected version will be a valid u32");

    info!("Using expected version: {expected_version}");

    let version = bios::get_latest_version().await;

    info!("Retrieved new version in {:?}", start.elapsed());

    let version = version?;
    let should_notify = version > expected_version;

    if should_notify {
        info!("Notifiers are being initialized! Version {version} > {expected_version}");

        let body = format!(
            "There's a new BIOS update for the ASUS B450-I: {expected_version} => {version}"
        );

        let (discord_response, email_response) =
            futures::join!(send_discord_msg(body.clone()), send_email(body));

        debug!("Finished trying to send notifications");

        let mut num_errors = 0;

        if let Err(discord_error) = discord_response {
            error!("Discord notification failed: {discord_error}");
            num_errors += 1;
        }

        if let Err(email_error) = email_response {
            error!("Email notification failed: {email_error}");
            num_errors += 1;
        }

        if num_errors == 2 {
            return Err(Error::from("All notifiers failed"));
        }
    } else if version < expected_version {
        warn!("Latest version is less than expected: {version} < {expected_version}");
    }

    let response = Response {
        req_id: request.context.request_id,
        expected_version,
        latest_version: version,
        notification_sent: should_notify,
    };

    info!(
        "Lambda function completed successfully in {:?}: {:?}",
        start.elapsed(),
        response
    );

    Ok(response)
}

async fn send_email(body: String) -> Result<()> {
    let email_notifier = EmailNotifier::init().await?;

    let message = EmailMessage {
        destination: EmailContact {
            name: Some(String::from("Bryan Hitchcock")),
            email: String::from("bryanhitc@gmail.com"),
        },
        subject: String::from("ASUS B450 ITX Bios Update"),
        body,
    };

    email_notifier.notify(message).await
}

async fn send_discord_msg(body: String) -> Result<()> {
    let discord_notify = DiscordNotifier::init().await?;
    discord_notify.notify(body).await
}
