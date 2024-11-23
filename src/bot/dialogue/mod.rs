use anyhow::Result;
use teloxide::{
    dispatching::{
        dialogue::{serializer::Json, GetChatId, RedisStorage},
        UpdateHandler,
    },
    prelude::*,
};

use super::{
    types::{CallbackData, CallbackOperation},
    Bot, BotDialogue, GlobalState,
};

pub mod ban_user;
pub mod decline_photo;
pub mod types;

async fn cancel_callback(bot: Bot, callback: CallbackQuery, dialogue: BotDialogue) -> Result<()> {
    let str_data = match &callback.data {
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

    if let CallbackOperation::Cancel = data.operation {
        dialogue.update(GlobalState::Idle).await?;

        bot.answer_callback_query(callback.id.clone())
            .text(t!("messages.operation_canceled"))
            .await?;
        bot.delete_message(callback.chat_id().unwrap(), callback.message.unwrap().id()).await?;
    };

    Ok(())
}

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    dptree::entry().branch(ban_user::scheme()).branch(decline_photo::scheme()).branch(
        Update::filter_callback_query()
            .enter_dialogue::<CallbackQuery, RedisStorage<Json>, GlobalState>()
            .branch(dptree::case![GlobalState::BanUser(x)].endpoint(cancel_callback))
            .branch(dptree::case![GlobalState::DeclinePhoto(x)].endpoint(cancel_callback)),
    )
}
