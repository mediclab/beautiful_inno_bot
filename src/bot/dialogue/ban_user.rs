use crate::bot::{traits::DialogueContext, Bot, BotDialogue, GlobalState};
use crate::db::entity::prelude::Ban;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{
        dialogue::{serializer::Json, RedisStorage},
        UpdateHandler,
    },
    prelude::*,
};

use super::types::BanUser;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum State {
    #[default]
    Reason,
}

async fn set_reason(bot: Bot, msg: Message, dialogue: BotDialogue) -> Result<()> {
    let user_id = match &msg.from {
        Some(f) => f.id.0 as i64,
        None => {
            error!("User is empty");

            return Ok(());
        }
    };

    if let Some(mut state) = BanUser::get(user_id).await {
        state.reason = msg.text().map(str::to_string);

        Ban::user(state.user_id, &state.reason.unwrap()).await;

        bot.send_message(msg.chat.id, t!("messages.user_banned")).await?;

        dialogue.update(GlobalState::Idle).await?;
    }

    Ok(())
}

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    dptree::entry().branch(
        Update::filter_message()
            .enter_dialogue::<Message, RedisStorage<Json>, GlobalState>()
            .branch(dptree::case![GlobalState::BanUser(x)].branch(dptree::case![State::Reason].endpoint(set_reason))),
    )
}
