# 梨窝（lily-nest）

> 梨梨的个人网站：项目展示、博客与技术分享

## 项目预览
- www.sulyhub.cn

## 项目简介
梨窝是一个基于 Rust + Axum 的个人主页/作品集网站，支持项目动态加载、团队成员展示、主题切换等功能，界面采用 Material You 风格，支持响应式设计。

## 技术栈
- Rust 2024
- [Axum](https://github.com/tokio-rs/axum) Web 框架
- Tokio 异步运行时
- Serde/TOML 配置
- Tower HTTP 静态资源服务
- 前端：Tailwind CSS（CDN）、自定义 Material 3 风格 CSS

## 主要功能
- 首页动态渲染（项目、团队成员、关于我）
- 配置文件驱动（config.toml、projects.toml）
- RESTful API（/api/v1/health, /api/v1/home/profile）
- 静态资源服务（图片、CSS、robots.txt、sitemap.xml 等）
- 主题切换（浅色/深色/跟随系统）
- 健康检查接口

## 项目结构
```
lily-nest/
├── Cargo.toml
├── config.toml         # 站点基础配置
├── projects.toml       # 项目列表配置
├── src/                # Rust 源码
│   ├── app.rs          # 应用入口与页面渲染
│   ├── config.rs       # 配置加载
│   ├── main.rs         # 启动入口
│   ├── model.rs        # 数据结构
│   └── routes/         # 路由
│       ├── api.rs      # API 路由
│       └── mod.rs
├── static/             # 静态资源
│   ├── css/MD3.css     # Material 3 风格 CSS
│   ├── js/index.js     # Index.html所需的 JS
│   └── images/         # 图片资源
├── templates/
│   └── index.html      # 首页模板
├── certs/
│   └── example.com.pem # SSL证书
│   └── example.com.key # SSL密钥
└── ...
```

## 启动方式
1. 安装 Rust（建议最新版）
2. 克隆本仓库并进入目录
3. 安装依赖并运行：
   ```bash
   cargo run
   ```
4. 无证书访问 [http://[::1]:8880](http://[::1]:8880)
5. 有证书访问 [https://[::1]:8443](https://[::1]:8443)

## 配置说明
- `config.toml`：站点基础信息、团队成员、关于我等
- `projects.toml`：项目列表
- `static/`：静态资源（图片、CSS、robots.txt等）

## 亮点与注意事项
- 支持热加载（debug模式下每次请求自动渲染最新页面）
- 生产环境建议用 Nginx 代理静态资源
- 仅供个人学习/展示用途，欢迎二次开发

## License
MIT
