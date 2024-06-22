use crate::db::entity::users::Model as Users;
use teloxide::types::User;

pub trait CanMention {
    fn mention_or_url(&self) -> String;
}

impl CanMention for Users {
    fn mention_or_url(&self) -> String {
        match &self.username {
            Some(uname) => format!("@{uname}"),
            None => format!("<a href=\"tg://user/?id={}\">{}</a>", self.user_id, self.firstname),
        }
    }
}

impl CanMention for User {
    fn mention_or_url(&self) -> String {
        match &self.username {
            Some(uname) => format!("@{uname}"),
            None => format!("<a href=\"{}\">{}</a>", self.url(), self.first_name),
        }
    }
}
