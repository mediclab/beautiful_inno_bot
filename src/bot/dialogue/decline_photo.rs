use crate::{
    bot::{Bot, BotDialogue, GlobalState, traits::DialogueContext},
    redis::{RedisManager, types::QueueMessage},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use teloxide::{
    dispatching::{
        UpdateHandler,
        dialogue::{GetChatId, RedisStorage, serializer::Json},
    },
    prelude::*,
    types::MessageId,
};

use super::types::DeclinePhoto;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum State {
    #[default]
    Reason,
}

#[allow(dead_code)]
#[allow(unused_variables)]
async fn set_reason(bot: Bot, msg: Message, dialogue: BotDialogue) -> Result<()> {
    let user_id = match &msg.from {
        Some(f) => f.id.0 as i64,
        None => {
            error!("User is empty");

            return Ok(());
        }
    };

    if let Some(mut state) = DeclinePhoto::get(user_id).await {
        state.reason = msg.text().map(str::to_string);

        let redis = RedisManager::global();

        redis
            .add_queue_item(&json!(QueueMessage::decline(state.photo_id, state.reason.unwrap_or_default())))
            .await;

        dialogue.update(GlobalState::Idle).await?;

        let photo = crate::db::entity::prelude::Photos::get_by_id(state.photo_id).await.unwrap();
        bot.delete_message(msg.chat_id().unwrap(), MessageId(photo.msg_id.unwrap() as i32))
            .await?;
    }

    Ok(())
}

#[allow(dead_code)]
pub fn scheme() -> UpdateHandler<anyhow::Error> {
    dptree::entry().branch(
        Update::filter_message()
            .enter_dialogue::<Message, RedisStorage<Json>, GlobalState>()
            .branch(dptree::case![GlobalState::DeclinePhoto(x)].branch(dptree::case![State::Reason].endpoint(set_reason))),
    )
}
