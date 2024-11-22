use crate::bot::BotConfig;
use crate::redis::subscriber::MessageHandler;
use backon::{ConstantBuilder, Retryable};
use envconfig::Envconfig;
use once_cell::sync::OnceCell;
use redis::AsyncCommands;
use redis::{aio::MultiplexedConnection, Client as RedisClient};
use redis_work_queue::{Item, KeyPrefix, WorkQueue};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::fmt::{Debug, Formatter};
use std::time::Duration;
use types::QueueMessage;

mod subscriber;
pub mod types;

pub static INSTANCE: OnceCell<RedisManager> = OnceCell::new();

#[derive(Envconfig, Clone, Debug)]
pub struct RedisConfig {
    #[envconfig(from = "REDIS_URL")]
    pub url: String,
}

pub struct RedisManager {
    client: RedisClient,
    queue: WorkQueue,
}

impl RedisManager {
    pub fn new(config: &RedisConfig) -> Self {
        let client = RedisClient::open(config.url.clone()).expect("Redis is not connected");
        Self {
            client,
            queue: WorkQueue::new(KeyPrefix::from("message_queue")),
        }
    }

    pub fn global() -> &'static RedisManager {
        INSTANCE.get().expect("RedisManager is not initialized")
    }

    pub async fn add_queue_item(&self, item: &Value) {
        let json_item = Item::from_string_data(item.to_string());
        let mut con = self.get_async_connection().await;

        match self.queue.add_item(&mut con, &json_item).await {
            Ok(_) => (),
            Err(e) => error!("Can't add queue: {}", e),
        }
    }

    async fn get_async_connection(&self) -> MultiplexedConnection {
        self.client.get_multiplexed_async_connection().await.expect("Can't get connection")
    }

    pub async fn get_model<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let ans: Option<String> = self.get_by_key(key).await;
        let val = serde_json::from_str::<T>(&ans.unwrap());

        match val {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    pub async fn set_model<T>(&self, key: &str, value: T) -> bool
    where
        T: Serialize,
    {
        self.set_by_key(key, &json!(value).to_string()).await
    }

    pub async fn get_by_key(&self, key: &str) -> Option<String> {
        let mut conn = self.get_async_connection().await;

        conn.get(key).await.unwrap_or(None)
    }

    pub async fn set_by_key(&self, key: &str, value: &str) -> bool {
        let mut conn = self.get_async_connection().await;

        conn.set(key, value).await.unwrap_or(false)
    }

    pub async fn subscriber(&self, bot_config: &BotConfig) {
        tokio::task::spawn({
            let mut con = self.get_async_connection().await;
            let config = bot_config.clone();

            async move {
                let handler = &Box::new(MessageHandler::new(&config));
                let queue = WorkQueue::new(KeyPrefix::from("message_queue"));

                loop {
                    let job = queue.lease(&mut con, None, Duration::from_secs(5)).await.unwrap_or_else(|e| {
                        error!("Can't lease job: {e}");
                        None
                    });

                    if let Some(item) = job {
                        let message: QueueMessage = item.data_json().expect("Can't deserialize message");

                        info!("Try to process message...");
                        let c = (|| async { handler.handle(&message).await }).retry(ConstantBuilder::default()).await;

                        if let Err(e) = c {
                            error!("Error occurred while trying process message: {e}");
                        }

                        queue.complete(&mut con, &item).await.expect("Can't complete message");
                    }
                }
            }
        });
    }
}

impl Debug for RedisManager {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
