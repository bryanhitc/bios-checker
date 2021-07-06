use std::time::Instant;

use lambda_notifiers::notifiers::discord::DiscordNotifier;
use lambda_notifiers::notifiers::email::{EmailContact, EmailMessage, EmailNotifier};
use lambda_notifiers::Notifier;

use anyhow::Result;
use lambda_runtime::{handler_fn, Context, Error};
use log::{debug, info, warn, LevelFilter};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;

mod bios;

// This doesn't do anything. I'm too lazy to try and figure out how
// to get rid of the JSON payload requirement for lambda.
#[derive(Debug, Deserialize)]
struct Request {
    command: String,
}

#[derive(Debug, Serialize)]
struct Response {
    req_id: String,
    expected_version: u32,
    latest_version: u32,
    notification_sent: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .with_module_level("bios_checker", LevelFilter::Trace)
        .with_module_level("lambda_notifiers", LevelFilter::Trace)
        .init()?;

    let func = handler_fn(my_handler);
    lambda_runtime::run(func).await
}

async fn my_handler(request: Request, ctx: Context) -> Result<Response, Error> {
    let start = Instant::now();
    info!("Handling request: {:?}", request);

    let expected_version = std::env::var("LATEST_VER")?;
    let expected_version = expected_version
        .parse::<u32>()
        .expect("The expected version will be a valid u32");

    info!("Using expected version: {}", expected_version);

    let notifier_start = Instant::now();
    let version = bios::get_latest_version().await;

    info!("Retrieved new version in {:?}", notifier_start.elapsed());

    let version = version?;
    let should_notify = version > expected_version;

    if should_notify {
        info!(
            "Notifiers are being initialized! Version {} > {}",
            version, expected_version
        );

        let body = format!(
            "There's a new BIOS update for the ASUS B450-I: {} => {}",
            expected_version, version
        );

        // TODO: Determine what to do with potential errors. E.g., should we retry?
        // Should we still succeed if at least one notification made it?
        let _responses = futures::join!(send_discord_msg(body.clone()), send_email(body));

        debug!("Notifications sent!");
    } else if version < expected_version {
        warn!(
            "Latest version is less than expected: {} < {}",
            version, expected_version
        );
    }

    let resp = Response {
        req_id: ctx.request_id,
        expected_version,
        latest_version: version,
        notification_sent: should_notify,
    };

    info!(
        "Lambda function completed successfully in {:?}: {:?}",
        start.elapsed(),
        resp
    );

    Ok(resp)
}

async fn send_email(body: String) -> Result<()> {
    let email_notifier = EmailNotifier::init().await?;

    let destination = EmailContact {
        name: Some(String::from("Bryan Hitchcock")),
        email: String::from("bryanhitc@gmail.com"),
    };

    let message = EmailMessage {
        destination,
        subject: String::from("ASUS B450 ITX Bios Update"),
        body,
    };

    email_notifier.notify(message).await
}

async fn send_discord_msg(body: String) -> Result<()> {
    let discord_notify = DiscordNotifier::init().await?;
    discord_notify.notify(body).await
}
