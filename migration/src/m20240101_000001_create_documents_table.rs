use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Documents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Documents::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Documents::Uuid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Documents::Title).string().not_null())
                    .col(ColumnDef::new(Documents::Content).text().not_null())
                    .col(ColumnDef::new(Documents::Category).string().not_null())
                    .col(ColumnDef::new(Documents::Tags).json().not_null())
                    .col(
                        ColumnDef::new(Documents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Documents::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_documents_category")
                    .table(Documents::Table)
                    .col(Documents::Category)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_documents_created_at")
                    .table(Documents::Table)
                    .col(Documents::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Documents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Documents {
    Table,
    Id,
    Uuid,
    Title,
    Content,
    Category,
    Tags,
    CreatedAt,
    UpdatedAt,
}