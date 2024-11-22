use crate::bot::types::CallbackOperation;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct QueueMessage {
    pub id: Uuid,
    pub operation: CallbackOperation,
    pub reason: Option<String>,
}
