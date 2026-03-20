use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::{load_about_items, load_profile, load_projects};

pub struct AppState {
    pub html_cache: RwLock<String>,
}

pub fn build_app() -> Router {
    let api_routes = crate::routes::api::router();
    let state = Arc::new(AppState {
        html_cache: RwLock::new(render_index()),
    });

    Router::new()
        .route("/", get(handler_home_page))
        .with_state(state)
        .route("/index.html", get(|| async { Redirect::permanent("/") }))
        .nest("/api/v1", api_routes)
        .route_service("/robots.txt", ServeFile::new("./static/robots.txt"))
        .route_service("/BingSiteAuth.xml", ServeFile::new("./static/BingSiteAuth.xml"))
        .route_service("/sitemap.xml", ServeFile::new("./static/sitemap.xml"))
        .route_service("/favicon.ico", ServeFile::new("./static/favicon.ico"))
        .nest_service("/images", ServeDir::new("./static/images"))
        .nest_service("/css", ServeDir::new("./static/css"))
        .nest_service("/js", ServeDir::new("./static/js"))
}

fn render_index() -> String {
    let profile_data = load_profile();
    let projects_data = load_projects();
    let about_data = load_about_items();

    let mut html = std::fs::read_to_string("templates/index.html").unwrap_or_else(|_| {
        "<!doctype html><html><body><h1>templates/index.html not found</h1></body></html>"
            .to_string()
    });
    // 组装 members_html
    let members_html = profile_data
        .team_members
        .iter()
        .map(|m| format!(r#"<span class="m3-chip">{}</span>"#, html_escape(m)))
        .collect::<String>();

    // 2. 组装项目预览 HTML
    let projects_html = projects_data.items.iter().map(|proj| {
        format!(
            r#"
            <a href="{url}" target="_blank" class="m3-item-card p-6 flex flex-col md:flex-row items-start md:items-center gap-6 decoration-none block transition-all hover:brightness-95">
                <div class="w-14 h-14 rounded-2xl bg-[var(--m3-primary-container)] flex-shrink-0 flex items-center justify-center text-[var(--m3-on-primary-container)]">
                    <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path></svg>
                </div>
                <div class="flex-1">
                    <h3 class="text-xl font-bold text-[var(--m3-on-surface)] mb-1">{name}</h3>
                    <p class="text-[var(--m3-on-surface-variant)] text-sm">{desc}</p>
                </div>
            </a>
            "#,
            url = html_escape(&proj.url),
            name = html_escape(&proj.name),
            desc = html_escape(&proj.desc)
        )
    }).collect::<String>();

    let about_items_html = about_data.items.iter().map(|item| {
        format!(
            r#"
            <div class="group bg-[var(--m3-surface-container)] rounded-[28px] p-6 shadow-sm border border-[var(--m3-outline-variant)] hover:border-[var(--m3-primary)] hover:shadow-md transition-all duration-300">
                <div class="w-12 h-12 rounded-xl bg-[var(--m3-primary-container)] mb-4 overflow-hidden flex items-center justify-center">
                    <img src="{icon}" alt="{title}" class="w-7 h-7 object-contain group-hover:scale-110 transition-transform" />
                </div>
                <h3 class="text-xl font-bold text-[var(--m3-on-surface)] mb-2">{title}</h3>
                <p class="text-[var(--m3-on-surface-variant)] text-sm leading-relaxed">
                    {content}
                </p>
            </div>
            "#,
            icon = html_escape(&item.icon_url),
            title = html_escape(&item.title),
            content = html_escape(&item.content)
        )
    }).collect::<String>();

    // 注入 JSON
    let profile_data_json = serde_json::to_string(&profile_data).unwrap();

    // 替换占位符
    html = html.replace("{{title}}", &html_escape(&profile_data.current_identity));
    html = html.replace("{{avatar}}", &html_escape(&profile_data.avatar_url));
    html = html.replace("{{bg}}", &html_escape(&profile_data.bg_url));
    html = html.replace("{{ver}}", &html_escape(&profile_data.site_version));
    html = html.replace("{{members_html}}", &members_html);
    html = html.replace("{{intro}}", &html_escape(&profile_data.intro));
    html = html.replace("{{p_json}}", &profile_data_json);
    // 注入项目 HTML
    html = html.replace("{{projects_html}}", &projects_html);
    html = html.replace("{{about_items_html}}", &about_items_html);

    html
}

async fn handler_home_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if cfg!(debug_assertions) {
        // debug：每次请求都重算（开发爽）
        return Html(render_index());
    }
    // release：用缓存
    let cache = state.html_cache.read().await;
    Html(cache.clone())
}

// 简单转义：用于插入到 HTML 文本节点/属性里
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
