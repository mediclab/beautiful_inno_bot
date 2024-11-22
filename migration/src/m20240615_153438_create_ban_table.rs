use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Ban::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Ban::Uuid)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Ban::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Ban::Reason).string().null())
                    .col(ColumnDef::new(Ban::BannedAt).timestamp().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("ban_user_id_users_user_id_fkey")
                    .from(Ban::Table, Ban::UserId)
                    .to(Users::Table, Users::UserId)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Ban::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Ban {
    Table,
    Uuid,
    UserId,
    Reason,
    BannedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
}
