use mime::Mime;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use uuid::Uuid;

use crate::exif::ExifLoader;
use crate::image::Image;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum CallbackOperation {
    #[serde(rename = "a")]
    Approve,
    #[serde(rename = "d")]
    Decline,
    #[default]
    #[serde(rename = "c")]
    Cancel,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CallbackData {
    #[serde(rename = "op")]
    pub operation: CallbackOperation,
    #[serde(rename = "doc", skip_serializing_if = "Option::is_none")]
    pub document: Option<Uuid>,
}

impl CallbackData {
    pub fn new(operation: CallbackOperation) -> Self {
        Self { operation, document: None }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum FileType {
    Heic,
    Png,
    Jpeg,
}

impl FileType {
    pub fn get_extension(self) -> &'static str {
        match self {
            FileType::Heic => "heic",
            FileType::Png => "png",
            FileType::Jpeg => "jpg",
        }
    }
}

impl From<String> for FileType {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&str> for FileType {
    fn from(value: &str) -> Self {
        match Mime::from_str(value) {
            Ok(t) => match t.subtype().as_str() {
                "heic" | "heif" => FileType::Heic,
                "png" => FileType::Png,
                _ => FileType::Jpeg,
            },
            Err(_) => FileType::Jpeg,
        }
    }
}

impl From<Option<String>> for FileType {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(t) => t.into(),
            None => FileType::Jpeg,
        }
    }
}

impl From<&Option<String>> for FileType {
    fn from(value: &Option<String>) -> Self {
        match value {
            Some(t) => t.as_str().into(),
            None => FileType::Jpeg,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PhotoToUpload {
    file_type: FileType,
    doc_path: PathBuf,
    photo_path: PathBuf,
    jpeg_path: PathBuf,
    thumb_path: PathBuf,
}

impl PhotoToUpload {
    pub fn new(file_type: &FileType) -> Self {
        let photo_path = match file_type {
            FileType::Png => PathBuf::from(format!("/tmp/{}.{}", Uuid::new_v4(), FileType::Png.get_extension())),
            _ => PathBuf::from(format!("/tmp/{}.{}", Uuid::new_v4(), FileType::Jpeg.get_extension())),
        };

        Self {
            doc_path: PathBuf::from(format!("/tmp/{}.{}", Uuid::new_v4(), file_type.get_extension())),
            photo_path,
            jpeg_path: PathBuf::from(format!("/tmp/{}.{}", Uuid::new_v4(), FileType::Jpeg.get_extension())),
            thumb_path: PathBuf::from(format!("/tmp/{}.{}", Uuid::new_v4(), FileType::Jpeg.get_extension())),
            file_type: *file_type,
        }
    }

    pub fn get_exif_info(&self) -> Vec<String> {
        let mut messages: Vec<String> = Vec::with_capacity(5);

        if let Ok(exif_info) = ExifLoader::new(&self.doc_path) {
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
        };

        messages
    }

    pub fn convert(&self) -> Result<(), BotError> {
        if !self.doc_path.exists() {
            return Err(BotError::FileNotExists(format!("File {} not exists!", self.doc_path.to_string_lossy())));
        }

        if self.file_type == FileType::Heic {
            let out = Command::new("heif-convert")
                .args(["-q", "90"])
                .arg(&self.doc_path)
                .arg(&self.photo_path)
                .output();

            if let Ok(output) = out {
                if !output.status.success() {
                    error!("Convert failed: {:?}", output);

                    return Err(BotError::ConvertingFailed(format!("Converting failed: {}", output.status)));
                }

                debug!("{:?}", output);

                std::fs::copy(&self.photo_path, &self.jpeg_path).unwrap_or_default();
            }

            return Ok(());
        }

        std::fs::copy(&self.doc_path, &self.photo_path).unwrap_or_default();
        std::fs::copy(&self.doc_path, &self.jpeg_path).unwrap_or_default();

        Ok(())
    }

    pub fn check(&self) -> Result<(), BotError> {
        let mut img = Image::new(&self.photo_path);
        let file_metadata = std::fs::metadata(&self.photo_path);

        if let Err(e) = file_metadata {
            return Err(BotError::GetMetadataFailed(format!("Get metadata failed: {}", e)));
        }

        if file_metadata.unwrap().len() > 10 * 1024 * 1024 {
            info!("Photo is over 10 MB. Scailing on 0.5x");

            if !img.scale(0.5).save(&self.photo_path) {
                error!("Scaling failed!");
            }
        }

        let (width, height) = img.get_size();

        if width > 4000 || height > 4000 {
            let scale = img.get_scaling(4000);

            info!("Photo is over 4000 px. Scailing to {}x", &scale);

            if !img.scale(scale).save(&self.photo_path) {
                error!("Scaling failed!");
            }
        }

        Ok(())
    }

    pub fn photo(&self) -> &Path {
        let _ = self.check();

        &self.photo_path
    }

    pub fn converted(&self) -> &Path {
        &self.jpeg_path
    }

    pub fn document_path(&self) -> &Path {
        &self.doc_path
    }

    pub fn thumbnail(&self) -> &Path {
        let mut img = Image::new(&self.photo_path);
        img.resize(320).save(&self.thumb_path);

        &self.thumb_path
    }

    pub fn delete_all(&self) -> bool {
        if std::fs::remove_file(&self.doc_path).is_err() {
            return false;
        }

        if std::fs::remove_file(&self.photo_path).is_err() {
            return false;
        }

        if std::fs::remove_file(&self.jpeg_path).is_err() {
            return false;
        }

        if std::fs::remove_file(&self.thumb_path).is_err() {
            return false;
        }

        true
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum BotError {
    FileNotExists(String),
    GetMetadataFailed(String),
    ConvertingFailed(String),
}
