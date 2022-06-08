use std::time::Instant;

use anyhow::Result;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport,
    Tokio1Executor,
};
use log::info;

use crate::Notifier;

#[derive(Debug)]
pub struct EmailNotifier {
    pub sender: EmailContact,
    password: String,
}

#[derive(Debug)]
pub struct EmailMessage {
    pub destination: EmailContact,
    pub subject: String,
    pub body: String,
}

#[derive(Debug)]
pub struct EmailContact {
    pub name: Option<String>,
    pub email: String,
}

impl std::fmt::Display for EmailContact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} <{}>", name, self.email),
            None => write!(f, "{}", self.email),
        }
    }
}

#[async_trait::async_trait]
impl Notifier for EmailNotifier {
    type Message = EmailMessage;

    async fn init() -> Result<Box<Self>> {
        let sender = EmailContact {
            name: std::env::var("SMTP_NAME").ok(),
            email: std::env::var("SMTP_EMAIL")?,
        };

        let password = std::env::var("SMTP_PASSWORD")?;
        let notifier = Self { sender, password };

        Ok(Box::new(notifier))
    }

    async fn notify(&self, message: Self::Message) -> Result<()> {
        let start = Instant::now();
        let creds = Credentials::new(self.sender.email.clone(), self.password.clone());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")?
            .credentials(creds)
            .build();

        let email = lettre::Message::builder()
            .from(self.sender.to_string().parse()?)
            .to(message.destination.to_string().parse()?)
            .subject(message.subject)
            .body(message.body)?;

        info!("Sending email: {:?}", email);

        let response = mailer
            .send(email)
            .await
            .map(|_| ())
            .map_err(|err| anyhow::anyhow!(err));

        info!("[Email] Finished sending in {:?}", start.elapsed());

        response
    }

    async fn shutdown(self) -> Result<()> {
        Ok(())
    }
}
