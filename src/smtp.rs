use anyhow::Result;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};

pub(crate) async fn send_email(latest_version: u32) -> Result<()> {
    let email = std::env::var("SMTP_EMAIL")?;
    let password = std::env::var("SMTP_PASSWORD")?;
    let creds = Credentials::new(email.clone(), password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")?
        .credentials(creds)
        .build();

    let email = Message::builder()
        .from(format!("Bryan Bot <{}>", email).parse().unwrap())
        .to("Bryan Hitchcock <bryanhitc@gmail.com>".parse().unwrap())
        .subject("ASUS B450 ITX Bios Update")
        .body(format!(
            "There's a new BIOS update for the ASUS B450-I: {}",
            latest_version
        ))?;

    mailer.send(email).await?;

    Ok(())
}
