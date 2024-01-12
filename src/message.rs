use crate::bot::get_user_text;
use crate::Application;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{
    Document, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MessageKind,
};

pub struct MessageHandler {
    pub app: Arc<Application>,
    pub msg: Message,
}

const MAX_FILE_SIZE: u32 = 15 * 1024 * 1024;

impl MessageHandler {
    pub async fn handle(msg: Message, app: Arc<Application>) -> anyhow::Result<()> {
        let handler = Self { app, msg };

        if let MessageKind::Common(_) = handler.msg.kind {
            if handler.msg.chat.is_private() {
                handler.private().await?;
            }
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

                        self.app.bot
                            .send_message(self.msg.chat.id, "😔 Прости, я не могу обработать фотку больше 15 Мб. Кажется, это уже перебор.")
                            .await?;
                    }
                    _ => {
                        self.app.bot
                            .send_message(self.msg.chat.id, "😔 Прости, я не могу понять что это за тип файла. Кажется, это даже не картинка.")
                            .await?;
                    }
                }
            }

            return Ok(());
        }

        self.app.bot
            .send_message(self.msg.chat.id, "😔 Прости, я принимаю фотки только в виде документов. Так не будет потери качества, и люди смогут скачать хорошую картинку.")
            .await?;

        Ok(())
    }

    async fn send_to_moderation(&self, doc: &Document) -> anyhow::Result<()> {
        self.app
            .bot
            .send_document(
                ChatId(self.app.admin),
                InputFile::file_id(doc.to_owned().file.id),
            )
            .caption(format!(
                "Автор: {}",
                get_user_text(self.msg.from().unwrap())
            ))
            .reply_markup(InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("👍 Запостить", "approve"),
                InlineKeyboardButton::callback("👎 Отказать", "decline"),
            ]]))
            .await?;

        self.app.bot
            .send_message(self.msg.chat.id, "😻 Спасибо за фотки! Отправил их на модерацию. Ищи свои фотографии в канале в ближайшее время!")
            .await?;

        Ok(())
    }
}
