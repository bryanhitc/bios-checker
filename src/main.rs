use lambda_runtime::{handler_fn, Context, Error};
use serde::{Deserialize, Serialize};

mod bios;
mod smtp;

// This doesn't do anything. I'm too lazy to try and figure out how
// to get rid of the JSON payload requirement for lambda.
#[derive(Deserialize)]
struct Request {
    command: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    expected_version: u32,
    latest_version: u32,
    notification_sent: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(my_handler);
    lambda_runtime::run(func).await?;

    Ok(())
}

async fn my_handler(_: Request, ctx: Context) -> Result<Response, Error> {
    let expected_version = std::env::var("LATEST_VER").unwrap_or_else(|_| "4204".to_string());
    let expected_version = expected_version.parse::<u32>().unwrap();

    let version = bios::get_latest_version().await?;
    let should_notify = version > expected_version;

    if should_notify {
        smtp::send_email(version).await?;
    }

    let resp = Response {
        req_id: ctx.request_id,
        expected_version,
        latest_version: version,
        notification_sent: should_notify,
    };

    Ok(resp)
}
