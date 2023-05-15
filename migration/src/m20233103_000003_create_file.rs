use crate::m20233103_000002_create_library::Library;
use sea_orm_migration::prelude::*;
// CREATE TABLE public.files
// (
//     id VARCHAR PRIMARY KEY NOT NULL,
//     parent VARCHAR NOT NULL,
//     library_id VARCHAR NOT NULL,
//     path VARCHAR NOT NULL,
//     folder BOOLEAN NOT NULL,
//     last_update BIGINT NOT NULL,
//     title VARCHAR,
//     season VARCHAR,
//     episode REAL,
//     release_group VARCHAR,
//     size bigint
// )

// TABLESPACE pg_default;

// ALTER TABLE public.files
//     OWNER to postgres;

#[derive(Iden)]
enum File {
    Table,
    Id,
    Parent,
    #[iden = "library_id"]
    LibraryId,
    Path,
    Folder,
    #[iden = "last_update"]
    LastUpdate,
    Title,
    Season,
    Episode,
    #[iden = "release_group"]
    ReleaseGroup,
    Size,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(File::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(File::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(File::Parent).string().not_null())
                    .col(ColumnDef::new(File::LibraryId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from_tbl(File::Table)
                            .from_col(File::LibraryId)
                            .to_tbl(Library::Table)
                            .to_col(Library::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(File::Path).string().not_null())
                    .col(ColumnDef::new(File::Folder).boolean().not_null())
                    .col(ColumnDef::new(File::LastUpdate).timestamp().not_null())
                    .col(ColumnDef::new(File::Title).string())
                    .col(ColumnDef::new(File::Season).string())
                    .col(ColumnDef::new(File::Episode).integer())
                    .col(ColumnDef::new(File::ReleaseGroup).string())
                    .col(ColumnDef::new(File::Size).integer())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(File::Table).to_owned())
            .await
    }
}
