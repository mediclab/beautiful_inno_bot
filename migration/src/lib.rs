pub use sea_orm_migration::prelude::*;

mod m20240615_151855_create_users_table;
mod m20240615_151858_create_photos_table;
mod m20240615_153438_create_ban_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240615_151855_create_users_table::Migration),
            Box::new(m20240615_151858_create_photos_table::Migration),
            Box::new(m20240615_153438_create_ban_table::Migration),
        ]
    }
}
