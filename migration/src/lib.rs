pub use sea_orm_migration::prelude::*;

mod m20231307_000004_create_file_tokens;
mod m20233103_000001_create_user;
mod m20233103_000002_create_library;
mod m20233103_000003_create_file;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20233103_000001_create_user::Migration),
            Box::new(m20233103_000002_create_library::Migration),
            Box::new(m20233103_000003_create_file::Migration),
            Box::new(m20231307_000004_create_file_tokens::Migration),
        ]
    }
}
