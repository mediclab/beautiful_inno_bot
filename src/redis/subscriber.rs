use crate::bot::types::{CallbackOperation, FileType, PhotoToUpload};
use crate::bot::{BotConfig, BotManager};
use crate::db::entity::photos::Model;
use crate::db::entity::prelude::Photos;
use crate::redis::types::QueueMessage;
use crate::types::CanMention;
use anyhow::Result;
use teloxide::{
    prelude::*,
    types::{InputFile, InputMedia, InputMediaDocument},
};

#[derive(Clone, Debug)]
pub struct MessageHandler {
    bot_manager: BotManager,
}

impl MessageHandler {
    pub fn new(config: &BotConfig) -> Self {
        Self {
            bot_manager: BotManager::new(config),
        }
    }

    pub async fn handle(&self, message: &QueueMessage) -> Result<()> {
        if let Some(doc) = &Photos::get_by_id(message.id).await {
            match message.operation {
                CallbackOperation::Approve => {
                    self.approve(doc).await?;
                }
                CallbackOperation::Decline => {
                    self.decline(doc, &message.reason).await?;
                }
                _ => {}
            };
        } else {
            error!("Can't find photo by uuid = {}", message.id);
        }

        Ok(())
    }

    async fn approve(&self, model: &Model) -> Result<()> {
        let bot = self.bot_manager.get_bot();

        if model.is_approved {
            error!("Photo already approved {}", &model.uuid);

            return Ok(());
        }

        let file_type = FileType::from(&model.mime_type);
        let photo_to_upload = PhotoToUpload::new(&file_type);
        let original_path = photo_to_upload.document_path();

        if let Err(e) = self.bot_manager.download_doc(&model.file_id, original_path).await {
            error!("Error occurred: {:?}", e);

            return Ok(());
        }

        if let Err(e) = photo_to_upload.convert() {
            error!("Error occurred: {:?}", e);

            return Ok(());
        }

        let photo_path = photo_to_upload.photo();
        let original_converted_path = photo_to_upload.converted();
        let thumb_path = photo_to_upload.thumbnail();
        let mut captions = photo_to_upload.get_exif_info();
        captions.push(format!("👤 Автор: {}", model.user().await.mention_or_url()));

        let caption = captions.join("\n");
        let original = InputFile::file(original_path).file_name(format!("original.{}", file_type.get_extension()));
        let original_converted = InputFile::file(original_converted_path).file_name("converted_original.jpg");
        let photo = InputFile::file(photo_path);
        let thumb = InputFile::file(thumb_path);

        let msg = bot.send_photo(ChatId(self.bot_manager.get_group_id()), photo).caption(caption).await?;

        match file_type {
            FileType::Heic => {
                bot.send_media_group(
                    ChatId(self.bot_manager.get_group_id()),
                    vec![
                        InputMedia::Document(InputMediaDocument::new(original).thumbnail(thumb)),
                        InputMedia::Document(InputMediaDocument::new(original_converted)),
                    ],
                )
                .await?;
            }
            _ => {
                bot.send_document(ChatId(self.bot_manager.get_group_id()), original)
                    .thumbnail(thumb)
                    .await?;
            }
        }

        if !photo_to_upload.delete_all() {
            warn!("Not all files have been deleted!")
        }

        model.approve(msg.id.0).await;

        Ok(())
    }

    async fn decline(&self, model: &Model, reason: &Option<String>) -> Result<()> {
        let bot = self.bot_manager.get_bot();

        if let Some(r) = reason {
            bot.send_message(ChatId(model.user_id), t!("messages.photo_was_declined_by_reason", reason = r))
                .await?;
        } else {
            bot.send_message(ChatId(model.user_id), t!("messages.photo_was_declined")).await?;
        };

        Ok(())
    }
}
