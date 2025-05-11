use crate::Application;
use crate::bot::{Bot, BotManager};
use crate::db::entity::prelude::{Ban, Photos};
use std::sync::Arc;
use teloxide::{
    dispatching::{
        UpdateHandler,
        dialogue::{RedisStorage, serializer::Json},
    },
    macros::BotCommands,
    prelude::*,
};

use super::dialogue::ban_user::State;
use super::dialogue::types::BanUser;
use super::traits::DialogueContext;
use super::{BotDialogue, GlobalState};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды которые поддерживает бот:")]
pub enum BotCommand {
    #[command(description = "Информация о боте")]
    Help,
    #[command(description = "Старт")]
    Start,
    #[command(description = "Забанить", hide)]
    Ban,
}

pub struct CommandHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub msg: Message,
    pub dialogue: BotDialogue,
}

impl CommandHandler {
    #[tracing::instrument(skip_all)]
    pub async fn handle(bot: Bot, msg: Message, cmd: BotCommand, app: Arc<Application>, dialogue: BotDialogue) -> anyhow::Result<()> {
        let handler = Self { app, bot, msg, dialogue };

        if !handler.msg.chat.is_private() {
            return Ok(());
        }

        if Ban::exists(handler.msg.chat.id.0).await {
            return Ok(());
        }

        match cmd {
            BotCommand::Help => {
                handler.help().await?;
            }
            BotCommand::Start => {
                handler.start().await?;
            }
            BotCommand::Ban => {
                handler.ban().await?;
            }
        };

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn help(&self) -> anyhow::Result<()> {
        self.bot
            .send_message(self.msg.chat.id, format!("Версия бота: {}", &self.app.config.version))
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn start(&self) -> anyhow::Result<()> {
        self.bot.send_message(self.msg.chat.id, t!("messages.start_greeting")).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn ban(&self) -> anyhow::Result<()> {
        let manager = BotManager::global();
        let cmd_user = self.msg.from.as_ref().unwrap().id.0 as i64;

        if manager.admin_id != cmd_user {
            return Ok(());
        }

        if let Some(reply) = self.msg.reply_to_message() {
            let photo = Photos::get_by_msg_id(reply.id.0).await;

            let state = BanUser {
                user_id: photo.unwrap().user_id,
                ..Default::default()
            };

            if state.set(cmd_user).await {
                self.dialogue.update(GlobalState::BanUser(State::Reason)).await?;

                self.bot
                    .send_message(self.msg.chat.id, t!("messages.enter_ban_reason"))
                    .reply_markup(super::markups::get_cancel_markup())
                    .await?;
            };
        }

        self.bot.delete_message(self.msg.chat.id, self.msg.id).await?;

        Ok(())
    }
}

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    dptree::entry().branch(
        Update::filter_message()
            .enter_dialogue::<Message, RedisStorage<Json>, GlobalState>()
            .filter(|m: Message| m.chat.is_private())
            .filter_command::<BotCommand>()
            .endpoint(CommandHandler::handle),
    )
}
