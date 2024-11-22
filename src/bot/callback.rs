use crate::bot::{
    traits::DialogueContext,
    types::{CallbackData, CallbackOperation},
    Bot,
};
use crate::db::entity::{photos, prelude::Photos};
use crate::redis::{types::QueueMessage, RedisManager};
use anyhow::Result;
use serde_json::json;
use teloxide::{
    dispatching::{
        dialogue::{serializer::Json, GetChatId, RedisStorage},
        UpdateHandler,
    },
    prelude::*,
};

use super::{
    dialogue::{decline_photo::State, types::DeclinePhoto},
    BotDialogue, GlobalState,
};

pub struct CallbackHandler {
    pub bot: Bot,
    pub callback: CallbackQuery,
    pub dialogue: BotDialogue,
}

impl CallbackHandler {
    pub async fn handle(bot: Bot, callback: CallbackQuery, dialogue: BotDialogue) -> Result<()> {
        let handler = Self { bot, callback, dialogue };

        let str_data = match &handler.callback.data {
            Some(s) => s,
            None => {
                error!("Callback data is empty!");

                return Ok(());
            }
        };

        let data: CallbackData = match serde_json::from_str(str_data.as_ref()) {
            Ok(d) => d,
            Err(e) => {
                error!("Callback data deserializing failed: {e}");

                return Ok(());
            }
        };

        let photo;

        if let Some(doc) = &data.document {
            photo = match Photos::get_by_id(*doc).await {
                Some(ph) => ph,
                None => {
                    error!("No photo found");

                    return Ok(());
                }
            };
        } else {
            return Ok(());
        }

        match data.operation {
            CallbackOperation::Approve => {
                handler.approve(&photo).await?;

                let msg = handler.callback.message.unwrap();
                handler.bot.delete_message(msg.chat().id, msg.id()).await?;
            }
            CallbackOperation::Decline => {
                handler.decline(&photo).await?;
            }
            _ => {}
        };

        Ok(())
    }

    async fn approve(&self, photo_doc: &photos::Model) -> Result<()> {
        let redis = RedisManager::global();

        redis
            .add_queue_item(&json!(QueueMessage {
                id: photo_doc.uuid,
                operation: CallbackOperation::Approve,
                ..Default::default()
            }))
            .await;

        self.bot
            .answer_callback_query(self.callback.id.clone())
            .text("Отправил в очередь на постинг")
            .await?;

        Ok(())
    }

    async fn decline(&self, photo_doc: &photos::Model) -> Result<()> {
        let cmd_user = self.callback.from.id.0 as i64;
        let state = DeclinePhoto {
            photo_id: photo_doc.uuid,
            ..Default::default()
        };

        if state.set(cmd_user).await {
            self.dialogue.update(GlobalState::DeclinePhoto(State::Reason)).await?;

            self.bot
                .send_message(self.callback.chat_id().unwrap(), "Введите причину отказа:")
                .reply_markup(super::markups::get_cancel_markup())
                .await?;
        };

        self.bot.answer_callback_query(self.callback.id.clone()).await?;

        Ok(())
    }
}

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    dptree::entry().branch(
        Update::filter_callback_query()
            .enter_dialogue::<CallbackQuery, RedisStorage<Json>, GlobalState>()
            .branch(dptree::case![GlobalState::Idle].endpoint(CallbackHandler::handle)),
    )
}
