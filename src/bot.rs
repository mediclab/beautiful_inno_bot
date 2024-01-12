use teloxide::types::User;

pub struct PhotoToUpload {
    pub photo_path: String,
    pub doc_path: String,
}

pub fn get_user_text(user: &User) -> String {
    match &user.username {
        Some(uname) => format!("@{uname}"),
        None => format!("<a href=\"{}\">{}</a>", user.url(), user.first_name),
    }
}
