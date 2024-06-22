use crate::bot::types::CallbackOperation;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueueMessage {
    pub id: Uuid,
    pub operation: CallbackOperation,
}
