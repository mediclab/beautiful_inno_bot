use crate::bot::types::{CallbackData, CallbackOperation};
use crate::bot::{Bot, BotManager};
use crate::db::entity::prelude::{Ban, Photos, Users};
use crate::types::CanMention;
use serde_json::json;
use teloxide::prelude::*;
use teloxide::types::{Document, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MessageKind};

pub struct MessageHandler {
    pub bot: Bot,
    pub msg: Message,
}

const MAX_FILE_SIZE: u32 = 15 * 1024 * 1024;

impl MessageHandler {
    pub async fn handle(bot: Bot, msg: Message) -> anyhow::Result<()> {
        let handler = Self { bot, msg };

        if Ban::exists(handler.msg.chat.id.0).await {
            return Ok(());
        }

        if let Some(u) = handler.msg.from.as_ref() {
            Users::add(u.clone().into()).await;
        }

        if let MessageKind::Common(_) = handler.msg.kind {
            handler.private().await?;
        }

        Ok(())
    }

    async fn private(&self) -> anyhow::Result<()> {
        if let Some(doc) = self.msg.document() {
            if let Some(doc_mime) = doc.to_owned().mime_type {
                match doc_mime.type_() {
                    mime::IMAGE => {
                        if doc.to_owned().file.size < MAX_FILE_SIZE {
                            return self.send_to_moderation(doc).await;
                        }

                        self.bot
                            .send_message(
                                self.msg.chat.id,
                                "😔 Прости, я не могу обработать фотку больше 15 Мб. Кажется, это уже перебор.",
                            )
                            .await?;
                    }
                    _ => {
                        self.bot
                            .send_message(
                                self.msg.chat.id,
                                "😔 Прости, я не могу понять что это за тип файла. Кажется, это даже не картинка.",
                            )
                            .await?;
                    }
                }
            }

            return Ok(());
        }

        self.bot
            .send_message(self.msg.chat.id, "😔 Прости, я принимаю фотки только в виде документов. Так не будет потери качества, и люди смогут скачать хорошую картинку.")
            .await?;

        Ok(())
    }

    async fn send_to_moderation(&self, doc: &Document) -> anyhow::Result<()> {
        let bot = BotManager::global();
        let model = match Photos::add(self.msg.clone().into()).await {
            Some(m) => m,
            None => {
                error!("Photo not added");
                return Ok(());
            }
        };

        let msg = self
            .bot
            .send_document(ChatId(bot.get_admin_id()), InputFile::file_id(doc.to_owned().file.id))
            .caption(format!("Автор: {}", self.msg.from.as_ref().unwrap().mention_or_url()))
            .reply_markup(InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback(
                    "👍 Запостить",
                    json!(CallbackData {
                        operation: CallbackOperation::Approve,
                        document: model.uuid
                    })
                    .to_string(),
                ),
                InlineKeyboardButton::callback(
                    "👎 Отказать",
                    json!(CallbackData {
                        operation: CallbackOperation::Decline,
                        document: model.uuid
                    })
                    .to_string(),
                ),
            ]]))
            .await?;

        model.update_msg_id(msg.id.0).await;

        self.bot
            .send_message(
                self.msg.chat.id,
                "😻 Спасибо за фотки! Отправил их на модерацию. Ищи свои фотографии в канале в ближайшее время!",
            )
            .await?;

        Ok(())
    }
}
