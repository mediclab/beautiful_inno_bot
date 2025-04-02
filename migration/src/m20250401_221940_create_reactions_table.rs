use sea_orm_migration::prelude::{sea_query::extension::postgres::Type, *};
use sea_orm_migration::schema::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(ReactionType::Enum)
                    .values([ReactionType::Emoji, ReactionType::CustomEmoji, ReactionType::Paid])
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Reactions::Table)
                    .if_not_exists()
                    .col(pk_uuid(Reactions::Uuid).default(Expr::cust("gen_random_uuid()")))
                    .col(uuid(Reactions::PhotoUuid))
                    .col(ColumnDef::new(Reactions::Type).custom(Alias::new("\"ReactionType\"")).not_null())
                    .col(string_null(Reactions::Content))
                    .col(big_unsigned(Reactions::Count).default(0))
                    .col(timestamp(Reactions::CreatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("reactions_photo_unique_idx")
                    .table(Reactions::Table)
                    .col(Reactions::PhotoUuid)
                    .col(Reactions::Type)
                    .col(Reactions::Content)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("reactions_meme_uuid_fkey")
                    .from(Reactions::Table, Reactions::PhotoUuid)
                    .to(Photos::Table, Photos::Uuid)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().if_exists().table(Reactions::Table).to_owned()).await?;
        manager.drop_type(Type::drop().if_exists().name(ReactionType::Enum).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Reactions {
    Table,
    Uuid,
    PhotoUuid,
    Type,
    Content,
    Count,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Photos {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
pub enum ReactionType {
    #[sea_orm(iden = "ReactionType")]
    Enum,
    #[sea_orm(iden = "Emoji")]
    Emoji,
    #[sea_orm(iden = "CustomEmoji")]
    CustomEmoji,
    #[sea_orm(iden = "Paid")]
    Paid,
}
