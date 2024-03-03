use teloxide::types::User;

pub struct PhotoToUpload {
    pub photo_path: String,
    pub doc_path: String,
    pub jpeg_path: String,
}

impl PhotoToUpload {
    pub fn delete_all(&self) -> bool {
        if std::fs::remove_file(&self.doc_path).is_err() {
            return false;
        }

        if std::fs::remove_file(&self.photo_path).is_err() {
            return false;
        }

        if std::fs::remove_file(&self.jpeg_path).is_err() {
            return false;
        }

        true
    }
}

pub fn get_user_text(user: &User) -> String {
    match &user.username {
        Some(uname) => format!("@{uname}"),
        None => format!("<a href=\"{}\">{}</a>", user.url(), user.first_name),
    }
}
