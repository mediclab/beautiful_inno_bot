use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use teloxide::{
    net::Download,
    prelude::*,
    types::{InputFile, InputMedia, InputMediaDocument},
};
use tokio::fs::File;
use uuid::Uuid;

use crate::bot::PhotoToUpload;
use crate::exif::ExifLoader;
use crate::image::Image;
use crate::Application;

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

    async fn approve(&self) -> anyhow::Result<()> {
        let doc = self.callback.message.as_ref().unwrap().document().unwrap();
        let doc_mime = doc.mime_type.as_ref().unwrap().subtype().as_str();
        let doc_ext = match doc_mime {
            "png" => "png",
            _ => "jpg",
        };
        let doc_path = self.download_doc(&doc.to_owned().file.id).await?;
        let photo_path = format!("/tmp/{}.{}", Uuid::new_v4(), doc_ext);
        let jpeg_path = format!("/tmp/{}.{}", Uuid::new_v4(), doc_ext);
        let author = self.callback.message.as_ref().unwrap().caption().unwrap_or_default();

        let caption = match ExifLoader::new(doc_path.to_owned()) {
            Ok(exif_info) => {
                let mut messages: Vec<String> = Vec::with_capacity(5);

                if let Some(_lens) = exif_info.get_lens_model() {
                    // messages.push(format!("📸 Снято на: {}", lens))
                }

                if let Some(maker_model) = exif_info.get_maker_model() {
                    messages.push(format!("📸 Снято на: {}", maker_model))
                }

                if let Some(afp_info) = exif_info.get_photo_info_string() {
                    messages.push(format!("ℹ️ {}", afp_info))
                }

                if let Some(_software) = exif_info.get_software() {
                    // messages.push(format!("⚠️ Обработано в: {}", software))
                }

                // Add delimiter
                if !messages.is_empty() {
                    messages.push(String::new());
                }

                messages.push(format!("👤 {}", author));
                messages.join("\n")
            }
            Err(_) => format!("👤 {}", author),
        };

        let upload = match doc_mime {
            "heic" | "heif" => {
                let out = Command::new("heif-dec")
                    .args(["-q", "90"])
                    .arg(&doc_path)
                    .arg(&photo_path)
                    .output()
                    .expect("failed to execute process");

                if !out.status.success() {
                    error!("{:?}", out);

                    return Err(anyhow!("Convert HEIC file failed!"));
                }

                debug!("{:?}", out);

                std::fs::copy(Path::new(&photo_path), Path::new(&jpeg_path)).unwrap_or_default();

                PhotoToUpload {
                    photo_path,
                    doc_path,
                    jpeg_path,
                }
            }
            _ => {
                std::fs::copy(Path::new(&doc_path), Path::new(&photo_path)).unwrap_or_default();
                std::fs::copy(Path::new(&doc_path), Path::new(&jpeg_path)).unwrap_or_default();

                PhotoToUpload {
                    photo_path,
                    doc_path,
                    jpeg_path,
                }
            }
        };

        let mut img = Image::new(&upload.photo_path);

        if std::fs::metadata(&upload.photo_path)?.len() > 10 * 1024 * 1024 {
            info!("Photo is over 10 MB. Scailing on 0.5x");

            if !img.scale(0.5).save(&upload.photo_path) {
                error!("Scaling failed!");
            }
        }

        let (width, height) = img.get_size();

        if width > 4000 || height > 4000 {
            let scale = img.get_scaling(4000);

            info!("Photo is over 4000 px. Scailing to {}x", &scale);

            if !img.scale(scale).save(&upload.photo_path) {
                error!("Scaling failed!");
            }
        }

        let thump_path = format!("/tmp/{}.jpg", Uuid::new_v4());
        let thumb = if let Some(doc_thumb) = doc.to_owned().thumb {
            InputFile::file_id(doc_thumb.file.id)
        } else {
            img.resize(320).save(&thump_path);

            InputFile::file(Path::new(&thump_path))
        };

        self.app
            .bot
            .send_photo(
                ChatId(self.app.config.group_id),
                InputFile::file(Path::new(&upload.photo_path)),
            )
            .caption(caption)
            .await?;

        let doc_path = Path::new(&upload.doc_path);
        let doc_ext = doc_path
            .extension()
            .unwrap_or("heic".as_ref())
            .to_str()
            .unwrap_or_default();

        match doc_mime {
            "heic" | "heif" => {
                self.app
                    .bot
                    .send_media_group(
                        ChatId(self.app.config.group_id),
                        vec![
                            InputMedia::Document(
                                InputMediaDocument::new(
                                    InputFile::file(doc_path).file_name(format!("original.{}", doc_ext)),
                                )
                                .thumb(thumb),
                            ),
                            InputMedia::Document(InputMediaDocument::new(
                                InputFile::file(Path::new(&upload.jpeg_path)).file_name("converted_original.jpg"),
                            )),
                        ],
                    )
                    .await?;
            }
            _ => {
                self.app
                    .bot
                    .send_document(
                        ChatId(self.app.config.group_id),
                        InputFile::file(Path::new(&upload.doc_path)).file_name(format!("original.{}", doc_ext)),
                    )
                    .thumb(thumb)
                    .await?;
            }
        }

        if !upload.delete_all() {
            warn!("Not all files have been deleted!")
        }

        std::fs::remove_file(&thump_path).unwrap_or_default();

        self.app
            .bot
            .answer_callback_query(self.callback.id.clone())
            .text("Запостил")
            .await
            .ok();

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
