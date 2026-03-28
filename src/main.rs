mod app;
mod auth;
mod db;
mod media;
mod model;
mod routes;

use std::{env, net::SocketAddr, path::PathBuf, process::Command};

use axum_server::tls_rustls::RustlsConfig;

#[tokio::main]
async fn main() {
    let db_path = env::var("LILY_NEST_DB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_db_path());

    let db = db::Database::new(&db_path)
        .await
        .expect("初始化 SQLite 失败");
    let admin_password = db
        .get_admin_password()
        .await
        .expect("读取管理员口令失败");
    let auth = auth::AuthState::new(admin_password);
    let app = app::build_app(db, auth);

    if let Some((cert_path, key_path)) = tls_files_from_env() {
        let config = RustlsConfig::from_pem_file(&cert_path, &key_path)
            .await
            .expect("加载 TLS 证书失败");
        let addr: SocketAddr = "[::]:8443".parse().expect("解析地址失败");
        println!(">> 梨窝相册 已启动: https://{}", addr);
        println!(">> SQLite 数据库: {}", db_path.display());

        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .expect("Server error");
        return;
    }

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("Failed to bind address");
    let addr = listener.local_addr().expect("读取本地端口失败");
    let open_url = format!("http://127.0.0.1:{}", addr.port());

    println!(">> 梨窝相册 已启动: {}", open_url);
    println!(">> SQLite 数据库: {}", db_path.display());
    println!(">> 首次启动请在页面里设置管理员口令");
    try_open_browser(&open_url);

    axum::serve(listener, app).await.expect("Server error");
}

fn tls_files_from_env() -> Option<(String, String)> {
    let cert_path = env::var("LILY_NEST_TLS_CERT").ok()?;
    let key_path = env::var("LILY_NEST_TLS_KEY").ok()?;

    if !std::path::Path::new(&cert_path).exists() || !std::path::Path::new(&key_path).exists() {
        panic!(
            "TLS 已声明，但证书文件不存在: cert_path={}, key_path={}",
            cert_path, key_path
        );
    }

    Some((cert_path, key_path))
}

fn default_db_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let dir = home
            .join("Library")
            .join("Application Support")
            .join("LilyNest");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("gallery.sqlite3");
    }

    #[cfg(not(target_os = "macos"))]
    {
        let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        exe_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("gallery.sqlite3")
    }
}

fn try_open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();

    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = Command::new("xdg-open").arg(url).spawn();
}
