use std::sync::Arc;

use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
    },
    response::{IntoResponse, Response},
    routing::get,
};

use crate::{
    app::AppState,
    media,
    model::{
        ApiErrorResponse, AuthStatusResponse, DeletePhotoResponse, HealthResponse, LoginRequest,
        NewPhoto, PhotoSummary, SetupPasswordRequest, UpdatePhotoRequest, UpdatePhotoResponse,
        UploadPhotoResponse, UploadedPhotoItem,
    },
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/status", get(auth_status))
        .route("/auth/setup", axum::routing::post(setup_password))
        .route("/auth/login", axum::routing::post(login))
        .route("/auth/logout", axum::routing::post(logout))
        .route("/photos", get(list_photos).post(upload_photo))
        .route(
            "/photos/{id}",
            axum::routing::delete(delete_photo).patch(update_photo),
        )
        .route("/photos/{id}/content", get(photo_content))
        .route("/health", get(health_handler))
}

async fn auth_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Json<AuthStatusResponse> {
    Json(AuthStatusResponse {
        authenticated: state
            .auth
            .is_authenticated(extract_session_token(&headers).as_deref()),
        setup_required: state.auth.requires_setup(),
    })
}

async fn setup_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SetupPasswordRequest>,
) -> Result<Response, ApiError> {
    if !state.auth.requires_setup() {
        return Err(ApiError::bad_request("admin password already configured"));
    }

    let password = payload.password.trim().to_string();
    let confirm_password = payload.confirm_password.trim().to_string();

    if password.len() < 6 {
        return Err(ApiError::bad_request("admin password must be at least 6 characters"));
    }

    if password != confirm_password {
        return Err(ApiError::bad_request("password confirmation does not match"));
    }

    let persisted = state.db.set_admin_password_if_unset(password.clone()).await?;
    if !persisted {
        return Err(ApiError::bad_request("admin password already configured"));
    }

    state.auth.set_password_if_unset(password).map_err(ApiError::internal)?;
    let token = state.auth.create_session()?;

    let mut response = Json(AuthStatusResponse {
        authenticated: true,
        setup_required: false,
    })
    .into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&build_session_cookie(&token))
            .map_err(|_| ApiError::internal("failed to build session cookie"))?,
    );

    Ok(response)
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    if state.auth.requires_setup() {
        return Err(ApiError::bad_request("admin password has not been configured yet"));
    }

    if !state.auth.verify_password(payload.password.trim()) {
        return Err(ApiError::unauthorized("invalid admin password"));
    }

    let token = state.auth.create_session()?;
    let mut response = Json(AuthStatusResponse {
        authenticated: true,
        setup_required: false,
    })
    .into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&build_session_cookie(&token))
            .map_err(|_| ApiError::internal("failed to build session cookie"))?,
    );

    Ok(response)
}

async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(token) = extract_session_token(&headers) {
        state.auth.revoke_session(&token)?;
    }

    let mut response = Json(AuthStatusResponse {
        authenticated: false,
        setup_required: state.auth.requires_setup(),
    })
    .into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_static("lily_nest_session=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax"),
    );

    Ok(response)
}

async fn list_photos(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let items = state
        .db
        .list_photos()
        .await?
        .into_iter()
        .map(|photo| PhotoSummary {
            id: photo.id,
            title: photo.title,
            description: photo.description,
            tags: photo.tags,
            filename: photo.filename,
            mime_type: photo.mime_type,
            created_at: photo.created_at,
            byte_size: photo.data.len(),
            content_url: format!("/api/v1/photos/{}/content", photo.id),
        })
        .collect::<Vec<_>>();

    Ok(Json(serde_json::json!({ "items": items })))
}

async fn upload_photo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadPhotoResponse>), ApiError> {
    ensure_admin(&state, &headers)?;

    let mut title: Option<String> = None;
    let mut description = String::new();
    let mut uploads: Vec<IncomingUpload> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "title" => {
                title = Some(
                    field
                        .text()
                        .await
                        .map_err(|err| ApiError::bad_request(err.to_string()))?,
                );
            }
            "description" => {
                description = field
                    .text()
                    .await
                    .map_err(|err| ApiError::bad_request(err.to_string()))?;
            }
            "photo" => {
                let filename = field.file_name().map(ToOwned::to_owned);
                let mime_type = field.content_type().map(ToOwned::to_owned);
                let data = field
                    .bytes()
                    .await
                    .map_err(|err| ApiError::bad_request(err.to_string()))?
                    .to_vec();

                uploads.push(IncomingUpload {
                    filename,
                    mime_type,
                    data,
                });
            }
            _ => {}
        }
    }

    if uploads.is_empty() {
        return Err(ApiError::bad_request("missing photo file"));
    }

    let normalized_title = title.unwrap_or_default().trim().to_string();
    let normalized_description = description.trim().to_string();
    let multiple = uploads.len() > 1;
    let mut created = Vec::with_capacity(uploads.len());

    for (index, upload) in uploads.into_iter().enumerate() {
        let mime_type = upload
            .mime_type
            .ok_or_else(|| ApiError::bad_request("missing image content type"))?;

        if !mime_type.starts_with("image/") {
            return Err(ApiError::bad_request("only image uploads are supported"));
        }

        let final_title = derive_title(
            &normalized_title,
            upload.filename.as_deref(),
            multiple,
            index,
        );
        let optimized = media::optimize_image_for_storage(
            upload.filename,
            mime_type,
            upload.data,
        )
        .await
        .map_err(ApiError::bad_request)?;

        let photo = state
            .db
            .insert_photo(NewPhoto {
                title: final_title,
                description: normalized_description.clone(),
                tags: Vec::new(),
                filename: optimized.filename,
                mime_type: optimized.mime_type,
                data: optimized.data,
            })
            .await?;

        created.push(UploadedPhotoItem {
            id: photo.id,
            title: photo.title,
            content_url: format!("/api/v1/photos/{}/content", photo.id),
        });
    }

    Ok((
        StatusCode::CREATED,
        Json(UploadPhotoResponse { items: created }),
    ))
}

async fn delete_photo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<DeletePhotoResponse>, ApiError> {
    ensure_admin(&state, &headers)?;

    let deleted = state.db.delete_photo(id).await?;

    if !deleted {
        return Err(ApiError::not_found("photo not found"));
    }

    Ok(Json(DeletePhotoResponse { id, deleted }))
}

async fn update_photo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePhotoRequest>,
) -> Result<Json<UpdatePhotoResponse>, ApiError> {
    ensure_admin(&state, &headers)?;

    let title = payload.title.trim();
    if title.is_empty() {
        return Err(ApiError::bad_request("title cannot be empty"));
    }

    let photo = state
        .db
        .update_photo(
            id,
            title.to_string(),
            payload.description.trim().to_string(),
            normalize_tags(&payload.tags),
        )
        .await?
        .ok_or_else(|| ApiError::not_found("photo not found"))?;

    Ok(Json(UpdatePhotoResponse {
        item: PhotoSummary {
            id: photo.id,
            title: photo.title,
            description: photo.description,
            tags: photo.tags,
            filename: photo.filename,
            mime_type: photo.mime_type,
            created_at: photo.created_at,
            byte_size: photo.data.len(),
            content_url: format!("/api/v1/photos/{}/content", photo.id),
        },
    }))
}

async fn photo_content(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let photo = state
        .db
        .get_photo(id)
        .await?
        .ok_or_else(|| ApiError::not_found("photo not found"))?;

    let mut response = Response::new(Body::from(photo.data));
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&photo.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );

    Ok(response)
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }

    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }
}

impl From<String> for ApiError {
    fn from(value: String) -> Self {
        Self::internal(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

struct IncomingUpload {
    filename: Option<String>,
    mime_type: Option<String>,
    data: Vec<u8>,
}

fn derive_title(base_title: &str, filename: Option<&str>, multiple: bool, index: usize) -> String {
    if !base_title.is_empty() {
        if multiple {
            format!("{base_title} {}", index + 1)
        } else {
            base_title.to_string()
        }
    } else {
        filename
            .filter(|name| !name.trim().is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("未命名照片 {}", index + 1))
    }
}

fn ensure_admin(state: &Arc<AppState>, headers: &HeaderMap) -> Result<(), ApiError> {
    if state
        .auth
        .is_authenticated(extract_session_token(headers).as_deref())
    {
        Ok(())
    } else {
        Err(ApiError::unauthorized("admin login required"))
    }
}

fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;

    cookie_header.split(';').find_map(|part| {
        let trimmed = part.trim();
        let (name, value) = trimmed.split_once('=')?;
        if name == "lily_nest_session" && !value.is_empty() {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn build_session_cookie(token: &str) -> String {
    format!("lily_nest_session={token}; Path=/; HttpOnly; SameSite=Lax")
}

fn normalize_tags(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .take(16)
        .map(|tag| tag.chars().take(24).collect::<String>())
        .collect()
}
