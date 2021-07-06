use std::{sync::Arc, time::Instant};

use anyhow::Result;
use log::{error, info};
use serde::Deserialize;
use serenity::{model::prelude::*, prelude::*, Client};
use tokio::sync::mpsc::UnboundedSender;

use crate::Notifier;

#[derive(Deserialize, Debug, Clone)]
struct DiscordConfig {
    auth_token: String,
    channel_name: String,
}

impl DiscordConfig {
    fn load() -> Result<Self> {
        let token = std::env::var("DISCORD_AUTH_TOKEN")?;
        let channel_name =
            std::env::var("DISCORD_CHANNEL_NAME").unwrap_or_else(|_| String::from("notification"));

        let config = Self {
            channel_name,
            auth_token: token,
        };

        Ok(config)
    }
}

pub struct DiscordNotifier {
    data: Arc<RwLock<TypeMap>>,
    http: Arc<serenity::http::Http>,
}

impl DiscordNotifier {
    fn new(data: Arc<RwLock<TypeMap>>, http: Arc<serenity::http::Http>) -> Self {
        Self { data, http }
    }
}

#[async_trait::async_trait]
impl Notifier for DiscordNotifier {
    type Message = String;

    async fn init() -> Result<Box<Self>> {
        let start = Instant::now();
        let config = DiscordConfig::load()?;

        // TODO: Use OneShot channel if possible?
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Result<()>>();

        let mut client = Client::builder(config.auth_token.clone())
            .event_handler(Handler { start, config })
            .type_map_insert::<BotInitializedSender>(sender.clone())
            .await?;

        let notifier =
            DiscordNotifier::new(client.data.clone(), client.cache_and_http.http.clone());

        tokio::spawn(async move {
            if let Err(err) = client.start().await {
                // Unwrap is fine since we know the receiver is waiting for a response
                error!("[Discord] Failed to start client: {}", err);
                sender.send(Err(anyhow::anyhow!(err))).unwrap();
            }
        });

        // Unwrap should be fine since either the client start error
        // above or the client ready event will send us a message
        receiver.recv().await.unwrap()?;

        Ok(Box::new(notifier))
    }

    async fn notify(&self, message: Self::Message) -> Result<()> {
        let start = Instant::now();

        // This shouldn't be too big, so I'd rather clone instead
        // of locking while sending a long async message.
        let channels = {
            let data = self.data.read().await;
            data.get::<RegisterChannelIds>()
                .cloned()
                .expect("Discord notification channels should exist")
        };

        let num_notifications = channels.len();

        let notifications = channels.into_iter().map(|channel| {
            channel.send_message(&self.http, |m| {
                let message = format!("@everyone {}", message.clone());
                m.content(message);
                m
            })
        });

        let responses = futures::future::join_all(notifications).await;
        let num_errors = responses.into_iter().filter(|r| r.is_err()).count();

        info!(
            "[Discord] Finished notifying {} channels in {:?}",
            num_notifications,
            start.elapsed()
        );

        match num_errors {
            0 => Ok(()),
            _ => Err(anyhow::format_err!(
                "An error occurred for sending {}/{} messages",
                num_errors,
                num_notifications
            )),
        }
    }
}

struct BotInitializedSender;

impl TypeMapKey for BotInitializedSender {
    type Value = UnboundedSender<Result<()>>;
}

struct RegisterChannelIds;

impl TypeMapKey for RegisterChannelIds {
    type Value = Vec<ChannelId>;
}

struct Handler {
    start: Instant,
    config: DiscordConfig,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("[Discord] {} is connected!", ready.user.name);

        let guild_channels = ready
            .guilds
            .into_iter()
            .map(|guild| ctx.http.get_channels(guild.id().into()));

        let guild_channels = futures::future::join_all(guild_channels).await;

        let notification_channels = guild_channels
            .into_iter()
            .filter_map(|guild_channels| guild_channels.ok())
            .filter_map(|channels| {
                channels
                    .into_iter()
                    .find(|channel| channel.name().contains(&self.config.channel_name))
            })
            .collect::<Vec<_>>();

        let channel_ids = notification_channels
            .iter()
            .map(|channel| channel.id)
            .collect::<Vec<_>>();

        let channel_names = notification_channels
            .into_iter()
            .map(|channel| format!("{} ({})", channel.name(), channel.id))
            .collect::<Vec<_>>()
            .join(", ");

        info!(
            "[Discord] {} automatically registered to the following channels for notifications: {}",
            ready.user.name, channel_names,
        );

        {
            let mut data = ctx.data.write().await;
            data.insert::<RegisterChannelIds>(channel_ids);

            let sender = data.get_mut::<BotInitializedSender>().unwrap();
            sender
                .send(Ok(()))
                .expect("Notifying main thread that discord bot is ready should work");
        };

        info!(
            "[Discord] Finished initialization in {:?}",
            self.start.elapsed()
        );
    }
}
