use crate::m20233103_000001_create_user::User;
use sea_orm_migration::prelude::*;
// CREATE TABLE public.file_tokens
// (
//     id VARCHAR PRIMARY KEY NOT NULL,
//     ownder VARCHAR NOT NULL,
//     token VARCHAR NOT NULL
// )

// TABLESPACE pg_default;

// ALTER TABLE public.file_tokens
//     OWNER to postgres;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FileTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FileTokens::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FileTokens::Owner).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from_tbl(FileTokens::Table)
                            .from_col(FileTokens::Owner)
                            .to_tbl(User::Table)
                            .to_col(User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(FileTokens::Token).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FileTokens::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum FileTokens {
    Table,
    Id,
    Owner,
    Token,
}
