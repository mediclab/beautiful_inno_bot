use crate::exif::ExifLoader;
use crate::Application;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use teloxide::net::Download;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MessageKind, User};
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::fs::File;

struct PhotoToUpload {
    photo_path: String,
    doc_path: String,
}

pub struct MessageHandler {
    pub app: Arc<Application>,
    pub msg: Message,
}

impl MessageHandler {
    pub async fn handle(msg: Message, app: Arc<Application>) -> Result<()> {
        let handler = Self { app, msg };

        if let MessageKind::Common(_) = handler.msg.kind {
            if handler.msg.chat.is_private() {
                handler.private().await?;
            }
        }

        Ok(())
    }

    async fn private(&self) -> Result<()> {
        if let Some(doc) = self.msg.document() {
            if let Some(doc_mime) = doc.to_owned().mime_type {
                match doc_mime.type_() {
                    mime::IMAGE => {
                        if doc.to_owned().file.size < 15 * 1024 * 1024 {
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
                        } else {
                            self.app.bot
                                .send_message(self.msg.chat.id, "😔 Прости, я не могу обработать фотку больше 15 Мб. Кажется, это уже перебор.")
                                .await?;
                        }
                    }
                    _ => {
                        self.app.bot
                            .send_message(self.msg.chat.id, "😔 Прости, я не могу понять что это за тип файла. Кажется, это даже не картинка.")
                            .await?;
                    }
                }
            }
        } else {
            self.app.bot
                .send_message(self.msg.chat.id, "😔 Прости, я принимаю фотки только в виде документов. Так не будет потери качества, и люди смогут скачать хорошую картинку.")
                .await?;
        }

        Ok(())
    }
}

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
    pub async fn handle(msg: Message, cmd: BotCommand, app: Arc<Application>) -> Result<()> {
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

    async fn help(&self) -> Result<()> {
        self.app
            .bot
            .send_message(
                self.msg.chat.id,
                format!("Версия бота: {}", self.app.version),
            )
            .await?;

        Ok(())
    }

    async fn start(&self) -> Result<()> {
        self.app.bot
            .send_message(self.msg.chat.id, "🤟 Привет, иннополисянин!\n\nРад, что ты заглянул!\n\nПрисылай сюда свои фотки города в виде файлов, я их обработаю и после небольшой модерации я их выложу в канал:\nhttps://t.me/beautiful_innopolis\n\nНе переживай, все сделаю в лучшем виде! 👌")
            .await?;

        Ok(())
    }
}

pub struct CallbackHandler {
    pub app: Arc<Application>,
    pub callback: CallbackQuery,
}

impl CallbackHandler {
    pub async fn handle(callback: CallbackQuery, app: Arc<Application>) -> Result<()> {
        let handler = Self { app, callback };
        let data = handler.callback.data.clone().unwrap_or_default();

        match data.as_str() {
            "approve" => {
                handler.approve().await?;
            }
            "decline" => {
                handler.decline().await?;
            }
            _ => {}
        };

        let msg = handler.callback.message.unwrap();
        handler.app.bot.delete_message(msg.chat.id, msg.id).await?;

        Ok(())
    }

    async fn approve(&self) -> Result<()> {
        let doc = self.callback.message.as_ref().unwrap().document().unwrap();
        let doc_path = self.download_doc(&doc.to_owned().file.id).await?;
        let exif_info = ExifLoader::new(doc_path.to_owned());
        let caption = format!(
            "Снято на: {} {}",
            exif_info.get_maker(),
            exif_info.get_model()
        );

        let upload = match doc.mime_type.as_ref().unwrap().subtype().as_str() {
            "heic" | "heif" => {
                let photo_path = format!("{}_p.jpg", &doc_path);
                let out = Command::new("heif-convert")
                    .args(["-q", "100"])
                    .arg(&doc_path)
                    .arg(&photo_path)
                    .output()
                    .expect("failed to execute process");

                if !out.status.success() {
                    error!("{:?}", out);

                    return Err(anyhow!("Convert HEIC file failed!"));
                }

                debug!("{:?}", out);

                PhotoToUpload {
                    photo_path,
                    doc_path,
                }
            }
            _ => PhotoToUpload {
                photo_path: doc_path.to_owned(),
                doc_path,
            },
        };

        self.app
            .bot
            .send_document(
                ChatId(self.app.group_id),
                InputFile::file(PathBuf::from(&upload.doc_path)),
            )
            .await?;

        self.app
            .bot
            .send_photo(
                ChatId(self.app.group_id),
                InputFile::file(PathBuf::from(&upload.photo_path)),
            )
            .caption(caption)
            .await?;

        std::fs::remove_file(&upload.doc_path).unwrap_or_default();
        std::fs::remove_file(&upload.photo_path).unwrap_or_default();

        self.app
            .bot
            .answer_callback_query(self.callback.id.clone())
            .text("Запостил")
            .await?;

        Ok(())
    }

    async fn decline(&self) -> Result<()> {
        self.app
            .bot
            .answer_callback_query(self.callback.id.clone())
            .text("Удолил")
            .await?;

        Ok(())
    }

    async fn download_doc(&self, doc_id: &String) -> Result<String> {
        let doc = self.app.bot.get_file(doc_id).await?;
        let extension = Path::new(&doc.path).extension().unwrap_or_default();
        let path = format!(
            "/tmp/{}.{}",
            uuid::Uuid::new_v4(),
            extension.to_ascii_lowercase().to_str().unwrap_or_default()
        );
        let mut file = File::create(&path).await?;

        self.app.bot.download_file(&doc.path, &mut file).await?;
        sleep(Duration::from_millis(50)); // Sometimes downloading is very fast
        debug!("Filesize {path} is = {}", std::fs::metadata(&path)?.len());

        Ok(path)
    }
}

pub fn get_user_text(user: &User) -> String {
    match &user.username {
        Some(uname) => format!("@{uname}"),
        None => format!("<a href=\"{}\">{}</a>", user.url(), user.first_name),
    }
}
