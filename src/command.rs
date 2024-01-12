use std::sync::Arc;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;

use crate::Application;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Команды которые поддерживает бот:"
)]
pub enum BotCommand {
    #[command(description = "Информация о боте")]
    Help,
    #[command(description = "Старт")]
    Start,
}

pub struct CommandHandler {
    pub app: Arc<Application>,
    pub msg: Message,
}

impl CommandHandler {
    pub async fn handle(
        msg: Message,
        cmd: BotCommand,
        app: Arc<Application>,
    ) -> anyhow::Result<()> {
        let handler = Self { app, msg };

        if !handler.msg.chat.is_private() {
            return Ok(());
        }

        match cmd {
            BotCommand::Help => {
                handler.help().await?;
            }
            BotCommand::Start => {
                handler.start().await?;
            }
        };

        Ok(())
    }

    async fn help(&self) -> anyhow::Result<()> {
        self.app
            .bot
            .send_message(
                self.msg.chat.id,
                format!("Версия бота: {}", self.app.version),
            )
            .await?;

        Ok(())
    }

    async fn start(&self) -> anyhow::Result<()> {
        self.app.bot
            .send_message(self.msg.chat.id, "🤟 Привет, иннополисянин!\n\nРад, что ты заглянул!\n\nПрисылай сюда свои фотки города в виде файлов, я их обработаю и после небольшой модерации я их выложу в канал:\nhttps://t.me/beautiful_innopolis\n\nНе переживай, все сделаю в лучшем виде! 👌")
            .await?;

        Ok(())
    }
}
