#![warn(clippy::all)]

pub mod notifiers;

use anyhow::Result;

#[async_trait::async_trait]
pub trait Notifier {
    type Message;

    async fn init() -> Result<Box<Self>>;
    async fn notify(&self, message: Self::Message) -> Result<()>;
}
