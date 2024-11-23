use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum QueueOperation {
    #[default]
    Approve,
    Decline,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct QueueMessage {
    pub id: Uuid,
    pub operation: QueueOperation,
    pub reason: Option<String>,
}

impl QueueMessage {
    pub fn approve(uuid: Uuid) -> Self {
        Self {
            id: uuid,
            operation: QueueOperation::Approve,
            reason: None,
        }
    }

    pub fn decline(uuid: Uuid, reason: String) -> Self {
        Self {
            id: uuid,
            operation: QueueOperation::Decline,
            reason: Some(reason),
        }
    }
}
