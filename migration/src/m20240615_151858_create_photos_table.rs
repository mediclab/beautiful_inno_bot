use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Photos::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Photos::Uuid)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Photos::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Photos::MsgId).big_integer().null())
                    .col(ColumnDef::new(Photos::FileId).string().not_null())
                    .col(ColumnDef::new(Photos::MimeType).string().null())
                    .col(ColumnDef::new(Photos::IsApproved).boolean().not_null().default(false))
                    .col(ColumnDef::new(Photos::ChannelMsgId).big_integer().null())
                    .col(ColumnDef::new(Photos::CreatedAt).timestamp().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Photos::PostedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("photos_user_id_users_user_id_fkey")
                    .from(Photos::Table, Photos::UserId)
                    .to(Users::Table, Users::UserId)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Photos::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Photos {
    Table,
    Uuid,
    UserId,
    MsgId,
    FileId,
    MimeType,
    IsApproved,
    ChannelMsgId,
    CreatedAt,
    PostedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
}
