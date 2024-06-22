use crate::bot::{Bot, BotManager};
use crate::db::entity::prelude::{Ban, Photos};
use crate::Application;
use std::sync::Arc;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;

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
}

impl CommandHandler {
    pub async fn handle(bot: Bot, msg: Message, cmd: BotCommand, app: Arc<Application>) -> anyhow::Result<()> {
        let handler = Self { app, bot, msg };

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

    async fn help(&self) -> anyhow::Result<()> {
        self.bot
            .send_message(self.msg.chat.id, format!("Версия бота: {}", &self.app.config.version))
            .await?;

        Ok(())
    }

    async fn start(&self) -> anyhow::Result<()> {
        self.bot
            .send_message(self.msg.chat.id, "🤟 Привет, иннополисянин!\n\nРад, что ты заглянул!\n\nПрисылай сюда свои фотки города в виде файлов, я их обработаю и после небольшой модерации я их выложу в канал:\nhttps://t.me/beautiful_innopolis\n\nНе переживай, все сделаю в лучшем виде! 👌")
            .await?;

        Ok(())
    }

    async fn ban(&self) -> anyhow::Result<()> {
        let manager = BotManager::global();

        if manager.admin_id != self.msg.from.as_ref().unwrap().id.0 as i64 {
            return Ok(());
        }

        if let Some(reply) = self.msg.reply_to_message() {
            let photo = Photos::get_by_msg_id(reply.id.0).await;
            Ban::user(photo.unwrap().user_id, "Причина не указана").await;

            self.bot.send_message(self.msg.chat.id, "Пользователь забанен").await?;

            self.bot.delete_message(self.msg.chat.id, reply.id).await?;
        }

        self.bot.delete_message(self.msg.chat.id, self.msg.id).await?;

        Ok(())
    }
}
