use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Clone)]
pub struct NewPhoto {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub filename: Option<String>,
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PhotoRecord {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub filename: Option<String>,
    pub mime_type: String,
    pub data: Vec<u8>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PhotoSummary {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub filename: Option<String>,
    pub mime_type: String,
    pub created_at: String,
    pub byte_size: usize,
    pub content_url: String,
}

#[derive(Debug, Serialize)]
pub struct UploadPhotoResponse {
    pub items: Vec<UploadedPhotoItem>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct UploadedPhotoItem {
    pub id: i64,
    pub title: String,
    pub content_url: String,
}

#[derive(Debug, Serialize)]
pub struct DeletePhotoResponse {
    pub id: i64,
    pub deleted: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SetupPasswordRequest {
    pub password: String,
    pub confirm_password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub authenticated: bool,
    pub setup_required: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePhotoRequest {
    pub title: String,
    pub description: String,
    pub tags: String,
}

#[derive(Debug, Serialize)]
pub struct UpdatePhotoResponse {
    pub item: PhotoSummary,
}
