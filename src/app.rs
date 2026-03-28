use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::DefaultBodyLimit,
    http::{
        HeaderName, HeaderValue, Request,
        header::{
            CONTENT_SECURITY_POLICY, CONTENT_TYPE, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
            X_FRAME_OPTIONS,
        },
    },
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};

use crate::{auth::AuthState, db::Database};

const INDEX_HTML: &str = include_str!("../embedded/index.html");
const APP_CSS: &str = include_str!("../embedded/app.css");
const APP_JS: &str = include_str!("../embedded/app.js");
const FAVICON_SVG: &[u8] = include_bytes!("../static/images/favicon.svg");

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub auth: AuthState,
}

pub fn build_app(db: Database, auth: AuthState) -> Router {
    let state = Arc::new(AppState { db, auth });
    let api_routes = crate::routes::api::router();

    Router::new()
        .route("/", get(handler_home_page))
        .route("/index.html", get(|| async { Redirect::permanent("/") }))
        .route("/assets/app.css", get(handler_app_css))
        .route("/assets/app.js", get(handler_app_js))
        .route("/favicon.ico", get(handler_favicon))
        .nest("/api/v1", api_routes)
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024))
        .layer(middleware::from_fn(add_security_headers))
        .with_state(state)
}

async fn handler_home_page() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn handler_app_css() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/css; charset=utf-8")], APP_CSS).into_response()
}

async fn handler_app_js() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        APP_JS,
    )
        .into_response()
}

async fn handler_favicon() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        FAVICON_SVG,
    )
        .into_response()
}

async fn add_security_headers(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(
        REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'",
        ),
    );

    response
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use crate::db::Database;

    fn temp_db_path() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("lily-nest-{nanos}.sqlite3"))
    }

    #[tokio::test]
    async fn adds_security_headers_to_responses() {
        let db = Database::new(temp_db_path()).await.unwrap();
        let auth = crate::auth::AuthState::new(Some("test-password".to_string()));
        let response = super::build_app(db, auth)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response
                .headers()
                .get("x-content-type-options")
                .unwrap()
                .to_str()
                .unwrap(),
            "nosniff"
        );
        assert!(response.headers().contains_key("content-security-policy"));
    }
}
