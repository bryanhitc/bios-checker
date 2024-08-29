#![warn(clippy::all)]

pub mod notifiers;

use anyhow::Result;

pub trait Notifier {
    type Message;

    fn init() -> impl std::future::Future<Output = Result<Box<Self>>> + Send;
    fn notify(
        &self,
        message: Self::Message,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
    fn shutdown(self) -> impl std::future::Future<Output = Result<()>> + Send;
}
