use crate::bot::{
    types::{CallbackData, CallbackOperation},
    Bot,
};
use crate::db::entity::{photos, prelude::Photos};
use crate::redis::{types::QueueMessage, RedisManager};
use anyhow::Result;
use serde_json::json;
use teloxide::prelude::*;

pub struct CallbackHandler {
    pub bot: Bot,
    pub callback: CallbackQuery,
}

impl CallbackHandler {
    pub async fn handle(bot: Bot, callback: CallbackQuery) -> Result<()> {
        let handler = Self { bot, callback };
        let data: CallbackData =
            serde_json::from_str(&handler.callback.data.clone().unwrap_or_else(|| r#"{}"#.to_string()))?;

        let photo = match Photos::get_by_id(data.document).await {
            Some(ph) => ph,
            None => {
                error!("No photo found");

                return Ok(());
            }
        };

        match data.operation {
            CallbackOperation::Approve => {
                handler.approve(&photo).await?;
            }
            CallbackOperation::Decline => {
                handler.decline(&photo).await?;
            }
        };

        let msg = handler.callback.message.unwrap();
        handler.bot.delete_message(msg.chat().id, msg.id()).await?;

        Ok(())
    }

    async fn approve(&self, photo_doc: &photos::Model) -> Result<()> {
        let redis = RedisManager::global();

        redis
            .add_queue_item(&json!(QueueMessage {
                id: photo_doc.uuid,
                operation: CallbackOperation::Approve,
            }))
            .await;

        self.bot
            .answer_callback_query(self.callback.id.clone())
            .text("Отправил в очередь на постинг")
            .await
            .ok();

        Ok(())
    }

    async fn decline(&self, photo_doc: &photos::Model) -> Result<()> {
        let redis = RedisManager::global();

        redis
            .add_queue_item(&json!(QueueMessage {
                id: photo_doc.uuid,
                operation: CallbackOperation::Decline,
            }))
            .await;

        self.bot
            .answer_callback_query(self.callback.id.clone())
            .text("Удолил")
            .await?;

        Ok(())
    }
}
