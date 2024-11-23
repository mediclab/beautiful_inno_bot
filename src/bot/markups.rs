use serde_json::json;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::db::entity::photos::Model;

use super::types::{CallbackData, CallbackOperation};

pub fn get_cancel_markup() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        t!("buttons.cancel"),
        json!(CallbackData::new(CallbackOperation::Cancel)).to_string(),
    )]])
}

pub fn get_document_markup(model: &Model) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(
            t!("buttons.approve"),
            json!(CallbackData {
                operation: CallbackOperation::Approve,
                document: Some(model.uuid)
            })
            .to_string(),
        ),
        InlineKeyboardButton::callback(
            t!("buttons.decline"),
            json!(CallbackData {
                operation: CallbackOperation::Decline,
                document: Some(model.uuid)
            })
            .to_string(),
        ),
    ]])
}
