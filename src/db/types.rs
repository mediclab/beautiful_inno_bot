use crate::db::entity::reactions::ReactionType;
use sea_orm::Set;
use teloxide::types::{Message, ReactionCount, ReactionType as TgReactionType, User};

impl From<User> for super::entity::users::ActiveModel {
    fn from(value: User) -> Self {
        super::entity::users::ActiveModel {
            user_id: Set(value.id.0 as i64),
            username: Set(value.username),
            firstname: Set(value.first_name),
            lastname: Set(value.last_name),
            ..Default::default()
        }
    }
}

impl From<Message> for super::entity::photos::ActiveModel {
    fn from(value: Message) -> Self {
        let doc = value.document().unwrap();

        super::entity::photos::ActiveModel {
            user_id: Set(value.from.as_ref().unwrap().id.0 as i64),
            file_id: Set(doc.file.id.clone()),
            mime_type: Set(doc.mime_type.clone().map(|u| u.to_string())),
            ..Default::default()
        }
    }
}

pub struct Reactions {
    pub r#type: ReactionType,
    pub content: Option<String>,
    pub count: u64,
}

impl From<&ReactionCount> for Reactions {
    fn from(react: &ReactionCount) -> Self {
        match &react.r#type {
            TgReactionType::Emoji { emoji } => Reactions {
                r#type: ReactionType::Emoji,
                content: Some(emoji.clone()),
                count: react.total_count,
            },
            TgReactionType::CustomEmoji { custom_emoji_id } => Reactions {
                r#type: ReactionType::Emoji,
                content: Some(custom_emoji_id.clone()),
                count: react.total_count,
            },
            TgReactionType::Paid => Reactions {
                r#type: ReactionType::Paid,
                content: None,
                count: react.total_count,
            },
        }
    }
}

impl From<super::entity::reactions::Model> for TgReactionType {
    fn from(value: super::entity::reactions::Model) -> Self {
        match value.r#type {
            ReactionType::Emoji => TgReactionType::Emoji {
                emoji: value.content.unwrap(),
            },
            ReactionType::CustomEmoji => TgReactionType::CustomEmoji {
                custom_emoji_id: value.content.unwrap(),
            },
            ReactionType::Paid => TgReactionType::Paid,
        }
    }
}
