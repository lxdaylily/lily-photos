mod app;
mod config;
mod model;
mod routes;

use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // 构建应用（路由、静态资源等）
    let app = app::build_app();

    // 尝试加载 TLS 配置
    if let Some(tls) = config::load_tls_config() {
        // 检查证书文件是否存在
        if std::path::Path::new(&tls.cert_path).exists()
            && std::path::Path::new(&tls.key_path).exists()
        {
            // 地址（可以从 config 读取）
            // 端口（http使用8880 https使用8443）
            println!(">> 检测到证书 启用HTTPS服务");
            let config = RustlsConfig::from_pem_file(&tls.cert_path, &tls.key_path)
                .await
                .expect("加载TLS证书失败");
            let addr: SocketAddr = "[::]:8443".parse().expect("解析地址失败");
            println!(">> 梨窝 已启动: https://{}", addr);

            axum_server::bind_rustls(addr, config)
                .serve(app.into_make_service())
                .await
                .expect("Server error");
            return;
        }
    }

    // 回退到 HTTP
    let addr: SocketAddr = "[::]:8880".parse().expect("解析地址失败");
    println!(">> 梨窝 已启动: http://{}", addr);
    // 启动服务
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server error");
}
