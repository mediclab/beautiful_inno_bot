use crate::db::Database;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,
    pub username: Option<String>,
    pub firstname: String,
    pub lastname: Option<String>,
    pub created_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::ban::Entity")]
    Ban,
    #[sea_orm(has_many = "super::photos::Entity")]
    Photos,
}

impl Related<super::ban::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ban.def()
    }
}

impl Related<super::photos::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Photos.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn add(model: ActiveModel) -> bool {
        Entity::insert(model)
            .on_conflict(
                OnConflict::column(Column::UserId)
                    .update_columns([Column::Username, Column::Firstname, Column::Lastname])
                    .to_owned(),
            )
            .exec(Database::global().connection())
            .await
            .is_ok()
    }
}

impl Model {
    #[allow(dead_code)]
    pub async fn ban(&self) -> bool {
        super::ban::Entity::insert(super::ban::ActiveModel {
            user_id: Set(self.user_id),
            ..Default::default()
        })
        .exec(Database::global().connection())
        .await
        .is_ok()
    }
}
