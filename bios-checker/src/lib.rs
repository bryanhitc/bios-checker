mod bios;

use anyhow::anyhow;
use lambda_notifiers::{
    notifiers::{
        discord::DiscordNotifier,
        email::{EmailContact, EmailMessage, EmailNotifier},
    },
    Notifier,
};
use serde::Serialize;
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

async fn send_email(body: String) -> anyhow::Result<()> {
    let email_notifier = EmailNotifier::init().await?;
    email_notifier
        .notify(EmailMessage {
            destination: EmailContact {
                name: Some(String::from("Bryan Hitchcock")),
                email: String::from("bryanhitc@gmail.com"),
            },
            subject: String::from("ASUS B450 ITX Bios Update"),
            body,
        })
        .await?;
    email_notifier.shutdown().await
}

async fn send_discord_msg(body: String) -> anyhow::Result<()> {
    let discord = DiscordNotifier::init().await?;
    discord.notify(body).await?;
    discord.shutdown().await
}

#[derive(Debug, Serialize)]
pub struct Response {
    req_id: String,
    expected_version: u32,
    latest_version: u32,
    notification_sent: bool,
}

pub async fn check_bios_version(request_id: String) -> anyhow::Result<Response> {
    let start = Instant::now();
    info!("Handling request id: {:?}", request_id);

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
            return Err(anyhow!("All notifiers failed"));
        }
    } else if version < expected_version {
        warn!("Latest version is less than expected: {version} < {expected_version}");
    }

    let response = Response {
        req_id: request_id,
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
