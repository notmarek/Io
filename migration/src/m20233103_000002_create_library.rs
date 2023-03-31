use sea_orm_migration::prelude::*;

// CREATE TABLE public.libraries
// (
//     id VARCHAR PRIMARY KEY NOT NULL,
//     path VARCHAR NOT NULL,
//     depth INTEGER NOT NULL,
//     last_scan INTEGER NOT NULL
// )

// TABLESPACE pg_default;

// ALTER TABLE public.libraries
//     OWNER to postgres;


#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Library::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Library::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Library::Path).string().not_null())
                    .col(ColumnDef::new(Library::Depth).integer().not_null())
                    .col(ColumnDef::new(Library::LastScan).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Library::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Library {
    Table,
    Id,
    Path,
    Depth,
    #[iden = "last_scan"]
    LastScan,
}
