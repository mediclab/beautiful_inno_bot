use crate::bot::{Bot, BotManager};
use crate::db::entity::prelude::{Ban, Photos, Users};
use crate::types::CanMention;
use teloxide::{
    dispatching::{
        UpdateHandler,
        dialogue::{RedisStorage, serializer::Json},
    },
    prelude::*,
    types::{Document, InputFile, MessageKind},
};

use super::GlobalState;

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

                        self.bot.send_message(self.msg.chat.id, t!("messages.max_filesize_reached")).await?;
                    }
                    _ => {
                        self.bot.send_message(self.msg.chat.id, t!("messages.unknown_filetype")).await?;
                    }
                }
            }

            return Ok(());
        }

        self.bot.send_message(self.msg.chat.id, t!("messages.documents_only")).await?;

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
            .reply_markup(super::markups::get_document_markup(&model))
            .await?;

        model.update_msg_id(msg.id.0).await;

        self.bot.send_message(self.msg.chat.id, t!("messages.thanks_for_send")).await?;

        Ok(())
    }
}

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    dptree::entry().branch(
        Update::filter_message()
            .enter_dialogue::<Message, RedisStorage<Json>, GlobalState>()
            .filter(|m: Message| m.chat.is_private())
            .endpoint(MessageHandler::handle),
    )
}
