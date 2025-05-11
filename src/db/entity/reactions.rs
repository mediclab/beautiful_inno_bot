use crate::db::Database;
use crate::db::types::Reactions;
use sea_orm::Set;
use sea_orm::{entity::prelude::*, sea_query::OnConflict};

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Eq, Hash)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ReactionType")]
pub enum ReactionType {
    #[sea_orm(string_value = "Emoji")]
    Emoji,
    #[sea_orm(string_value = "CustomEmoji")]
    CustomEmoji,
    #[sea_orm(string_value = "Paid")]
    Paid,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "reactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub photo_uuid: Uuid,
    pub r#type: ReactionType,
    pub content: Option<String>,
    pub count: i64,
    pub created_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::photos::Entity",
        from = "Column::PhotoUuid",
        to = "super::photos::Column::Uuid",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Photos,
}

impl Related<super::photos::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Photos.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    #[tracing::instrument(skip_all)]
    pub async fn get_photos_reactions(photo_uuid: Uuid) -> Vec<Model> {
        let res = Self::find()
            .filter(Column::PhotoUuid.eq(photo_uuid))
            .all(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get reactions from database: {e}");

            Vec::new()
        })
    }

    #[tracing::instrument(skip_all)]
    pub async fn update_reactions(photo_uuid: Uuid, reactions: Vec<Reactions>) -> bool {
        Self::insert_many(reactions.into_iter().map(|r| ActiveModel {
            photo_uuid: Set(photo_uuid),
            r#type: Set(r.r#type),
            content: Set(r.content),
            count: Set(r.count as i64),
            ..Default::default()
        }))
        .on_conflict(
            OnConflict::columns([Column::PhotoUuid, Column::Type, Column::Content])
                .update_column(Column::Count)
                .to_owned(),
        )
        .exec(Database::global().connection())
        .await
        .is_ok()
    }

    #[tracing::instrument(skip_all)]
    pub async fn remove_reactions(reaction_uuids: Vec<Uuid>) -> bool {
        Self::delete_many()
            .filter(Column::Uuid.is_in(reaction_uuids))
            .exec(Database::global().connection())
            .await
            .is_ok()
    }
}
