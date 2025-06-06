use crate::db::{Database, entity::prelude::Reactions};
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{IntoActiveModel, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "photos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub user_id: i64,
    pub msg_id: Option<i64>,
    pub file_id: String,
    pub mime_type: Option<String>,
    pub is_approved: bool,
    pub channel_msg_id: Option<i64>,
    pub created_at: Option<DateTime>,
    pub posted_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::UserId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    #[tracing::instrument(skip_all)]
    pub async fn add(model: ActiveModel) -> Option<Model> {
        let res = model.insert(Database::global().connection()).await;

        match res {
            Ok(m) => Some(m),
            Err(e) => {
                error!("Can't add photo to database: {e}");
                None
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_by_id(uuid: Uuid) -> Option<Model> {
        let res = Self::find_by_id(uuid).one(Database::global().connection()).await;

        res.unwrap_or_else(|e| {
            error!("Can't get photo from database: {e}");
            None
        })
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_by_msg_id(msg_id: i32) -> Option<Model> {
        let res = Self::find().filter(Column::MsgId.eq(msg_id)).one(Database::global().connection()).await;

        res.unwrap_or_else(|e| {
            error!("Can't get photo from database: {e}");
            None
        })
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_by_channel_msg_id(msg_id: i32) -> Option<Model> {
        let res = Self::find()
            .filter(Column::ChannelMsgId.eq(msg_id))
            .one(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get photo from database: {e}");
            None
        })
    }
}

impl Model {
    #[tracing::instrument(skip_all)]
    pub async fn approve(&self, msg_id: i32) -> bool {
        let mut model = self.clone().into_active_model();
        model.is_approved = Set(true);
        model.posted_at = Set(Some(Utc::now().naive_utc()));
        model.channel_msg_id = Set(Some(msg_id as i64));

        Entity::update(model).exec(Database::global().connection()).await.is_ok()
    }

    #[tracing::instrument(skip_all)]
    pub async fn update_msg_id(&self, msg_id: i32) -> bool {
        let mut model = self.clone().into_active_model();
        model.msg_id = Set(Some(msg_id as i64));

        Entity::update(model).exec(Database::global().connection()).await.is_ok()
    }

    #[tracing::instrument(skip_all)]
    pub async fn user(&self) -> super::users::Model {
        super::users::Entity::find_by_id(self.user_id)
            .one(Database::global().connection())
            .await
            .expect("Can't get user")
            .unwrap()
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_reactions(&self) -> Vec<super::reactions::Model> {
        Reactions::get_photos_reactions(self.uuid).await
    }
}
