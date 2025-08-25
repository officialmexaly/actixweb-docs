use actix_web::{web, HttpResponse, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

use entity::documents::{
    CreateDocumentRequest, DocumentResponse, UpdateDocumentRequest,
    Entity as Documents, Model as Document, ActiveModel as ActiveDocument,
};
use crate::errors::{AppError, AppResult};

// Health check endpoint
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "message": "Tech Docs API is running"
    })))
}

// Get all documents with optional filtering
pub async fn get_documents(
    db: web::Data<DatabaseConnection>,
    query: web::Query<DocumentQuery>,
) -> AppResult<HttpResponse> {
    let mut select = Documents::find();
    
    // Apply category filter
    if let Some(category) = &query.category {
        if category != "all" {
            select = select.filter(entity::documents::Column::Category.eq(category));
        }
    }
    
    // Apply search filter (search in title and content)
    if let Some(search) = &query.search {
        let search_pattern = format!("%{}%", search);
        select = select.filter(
            entity::documents::Column::Title.like(&search_pattern)
                .or(entity::documents::Column::Content.like(&search_pattern))
        );
    }
    
    let documents: Vec<Document> = select
        .order_by_desc(entity::documents::Column::UpdatedAt)
        .all(db.get_ref())
        .await?;
    
    let response: Vec<DocumentResponse> = documents
        .into_iter()
        .map(DocumentResponse::from)
        .collect();
    
    Ok(HttpResponse::Ok().json(response))
}

// Get single document by UUID
pub async fn get_document(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let uuid = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid UUID format".to_string()))?;
    
    let document = Documents::find()
        .filter(entity::documents::Column::Uuid.eq(uuid))
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;
    
    Ok(HttpResponse::Ok().json(DocumentResponse::from(document)))
}

// Create new document
pub async fn create_document(
    db: web::Data<DatabaseConnection>,
    json: web::Json<CreateDocumentRequest>,
) -> AppResult<HttpResponse> {
    let request = json.into_inner();
    
    if request.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title cannot be empty".to_string()));
    }
    
    if request.content.trim().is_empty() {
        return Err(AppError::BadRequest("Content cannot be empty".to_string()));
    }
    
    let tags_json = serde_json::Value::Array(
        request.tags.into_iter()
            .map(|tag| serde_json::Value::String(tag))
            .collect()
    );
    
    let new_document = ActiveDocument {
        uuid: Set(Uuid::new_v4()),
        title: Set(request.title),
        content: Set(request.content),
        category: Set(request.category),
        tags: Set(tags_json.into()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    
    let document = new_document.insert(db.get_ref()).await?;
    
    Ok(HttpResponse::Created().json(DocumentResponse::from(document)))
}

// Update document
pub async fn update_document(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
    json: web::Json<UpdateDocumentRequest>,
) -> AppResult<HttpResponse> {
    let uuid = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid UUID format".to_string()))?;
    
    let document = Documents::find()
        .filter(entity::documents::Column::Uuid.eq(uuid))
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;
    
    let request = json.into_inner();
    let mut active_document: ActiveDocument = document.into();
    
    if let Some(title) = request.title {
        if title.trim().is_empty() {
            return Err(AppError::BadRequest("Title cannot be empty".to_string()));
        }
        active_document.title = Set(title);
    }
    
    if let Some(content) = request.content {
        if content.trim().is_empty() {
            return Err(AppError::BadRequest("Content cannot be empty".to_string()));
        }
        active_document.content = Set(content);
    }
    
    if let Some(category) = request.category {
        active_document.category = Set(category);
    }
    
    if let Some(tags) = request.tags {
        let tags_json = serde_json::Value::Array(
            tags.into_iter()
                .map(|tag| serde_json::Value::String(tag))
                .collect()
        );
        active_document.tags = Set(tags_json.into());
    }
    
    active_document.updated_at = Set(chrono::Utc::now());
    
    let updated_document = active_document.update(db.get_ref()).await?;
    
    Ok(HttpResponse::Ok().json(DocumentResponse::from(updated_document)))
}

// Delete document
pub async fn delete_document(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let uuid = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid UUID format".to_string()))?;
    
    let document = Documents::find()
        .filter(entity::documents::Column::Uuid.eq(uuid))
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;
    
    Documents::delete_by_id(document.id)
        .exec(db.get_ref())
        .await?;
    
    Ok(HttpResponse::NoContent().finish())
}

// Get all categories
pub async fn get_categories(
    db: web::Data<DatabaseConnection>,
) -> AppResult<HttpResponse> {
    let categories: Vec<String> = Documents::find()
        .select_only()
        .column(entity::documents::Column::Category)
        .distinct()
        .into_tuple()
        .all(db.get_ref())
        .await?;
    
    Ok(HttpResponse::Ok().json(categories))
}

// Query parameters for filtering
#[derive(serde::Deserialize)]
pub struct DocumentQuery {
    pub category: Option<String>,
    pub search: Option<String>,
}