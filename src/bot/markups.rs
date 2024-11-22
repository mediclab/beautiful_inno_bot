use serde_json::json;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use super::types::{CallbackData, CallbackOperation};

pub fn get_cancel_markup() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Отмена",
        json!(CallbackData::new(CallbackOperation::Cancel)).to_string(),
    )]])
}
