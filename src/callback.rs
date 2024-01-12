use anyhow::anyhow;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::InputFile;
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
    pub async fn handle(callback: CallbackQuery, app: Arc<Application>) -> anyhow::Result<()> {
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
        let doc_path = self.download_doc(&doc.to_owned().file.id).await?;
        let exif_info = ExifLoader::new(doc_path.to_owned());

        debug!("Debug fields: {}", exif_info.get_photo_info_string());

        let caption = format!(
            "📸 Снято на: {} {}\nℹ️ {}\n\n👤 {}",
            exif_info.get_maker(),
            exif_info.get_model(),
            exif_info.get_photo_info_string(),
            self.callback
                .message
                .as_ref()
                .unwrap()
                .caption()
                .unwrap_or_default()
        );

        let upload = match doc.mime_type.as_ref().unwrap().subtype().as_str() {
            "heic" | "heif" => {
                let photo_path = format!("{}_p.jpg", Uuid::new_v4());
                let out = Command::new("heif-convert")
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

                PhotoToUpload {
                    photo_path,
                    doc_path,
                }
            }
            _ => PhotoToUpload {
                photo_path: Uuid::new_v4().to_string(),
                doc_path,
            },
        };

        if std::fs::metadata(&upload.photo_path)?.len() > 10 * 1024 * 1024 {
            info!("Photo is over 10 MB. Scailing on 0.5x");

            let mut img = Image::new(&upload.photo_path);
            if !img.scale(0.5).save(&upload.photo_path) {
                error!("Scaling failed!");
            }
        }

        let mut img = Image::new(&upload.photo_path);
        let thump_path = format!("/tmp/{}.jpg", Uuid::new_v4());
        img.resize(320).save(&thump_path);

        self.app
            .bot
            .send_photo(
                ChatId(self.app.group_id),
                InputFile::file(Path::new(&upload.photo_path)),
            )
            .caption(caption)
            .await?;

        self.app
            .bot
            .send_document(
                ChatId(self.app.group_id),
                InputFile::file(Path::new(&upload.doc_path)),
            )
            .thumb(InputFile::file(Path::new(&thump_path)))
            .await?;

        std::fs::remove_file(&upload.doc_path).unwrap_or_default();
        std::fs::remove_file(&upload.photo_path).unwrap_or_default();
        std::fs::remove_file(&thump_path).unwrap_or_default();

        self.app
            .bot
            .answer_callback_query(self.callback.id.clone())
            .text("Запостил")
            .await?;

        Ok(())
    }

    async fn decline(&self) -> anyhow::Result<()> {
        self.app
            .bot
            .answer_callback_query(self.callback.id.clone())
            .text("Удолил")
            .await?;

        Ok(())
    }

    async fn download_doc(&self, doc_id: &String) -> anyhow::Result<String> {
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
