use lambda_runtime::{handler_fn, Context, Error};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use tokio::time::Instant;

mod bios;
mod smtp;

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
    SimpleLogger::new().init()?;
    info!("SimpleLogger initialized. Starting lambda function...");

    let now = Instant::now();
    let func = handler_fn(my_handler);
    let response = lambda_runtime::run(func).await;

    if let Err(err) = response {
        error!("Error occurred while executing function: {:?}", err);
        return Err(err);
    }

    info!(
        "Lambda function completed successfully in {:?}",
        now.elapsed()
    );
    response
}

async fn my_handler(request: Request, ctx: Context) -> Result<Response, Error> {
    println!("Handling request: {:?}", request);

    let expected_version = std::env::var("LATEST_VER").unwrap_or_else(|_| "4204".to_string());
    let expected_version = expected_version.parse::<u32>().unwrap();

    info!("Using expected version: {}", expected_version);

    let version = bios::get_latest_version().await?;
    let should_notify = version > expected_version;

    if should_notify {
        smtp::send_email(version).await?;
        info!(
            "Email sent successfully! Version {} > {}",
            version, expected_version
        );
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

    info!("Response: {:?}", resp);
    Ok(resp)
}
