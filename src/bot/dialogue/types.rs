use crate::bot::traits::DialogueContext;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct BanUser {
    pub user_id: i64,
    pub reason: Option<String>,
}

impl DialogueContext for BanUser {}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DeclinePhoto {
    pub photo_id: uuid::Uuid,
    pub reason: Option<String>,
}

impl DialogueContext for DeclinePhoto {}
