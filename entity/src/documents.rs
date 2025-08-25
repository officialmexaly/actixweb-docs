use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "documents")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    
    #[sea_orm(unique)]
    pub uuid: Uuid,
    
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Json,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: i32,
    pub uuid: Uuid,
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

impl From<Model> for DocumentResponse {
    fn from(model: Model) -> Self {
        let tags = match model.tags {
            Json::Array(arr) => arr.into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![],
        };

        Self {
            id: model.id,
            uuid: model.uuid,
            title: model.title,
            content: model.content,
            category: model.category,
            tags,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}