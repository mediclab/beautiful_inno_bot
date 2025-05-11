use crate::db::Database;
use sea_orm::Set;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "ban")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub user_id: i64,
    pub reason: Option<String>,
    pub banned_at: Option<DateTime>,
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
    pub async fn user(user_id: i64, reason: &str) -> bool {
        Entity::insert(ActiveModel {
            user_id: Set(user_id),
            reason: Set(Some(reason.to_string())),
            ..Default::default()
        })
        .exec(Database::global().connection())
        .await
        .is_ok()
    }

    #[tracing::instrument(skip_all)]
    pub async fn exists(user_id: i64) -> bool {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .count(Database::global().connection())
            .await
            .unwrap_or(0)
            > 0
    }
}
