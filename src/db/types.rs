use sea_orm::Set;
use teloxide::types::{Message, User};

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
