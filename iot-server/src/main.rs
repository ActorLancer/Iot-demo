use axum::Router;
use dotenv::dotenv;
use std::sync::Arc;
// 引入中间件依赖，处理 CORS 问题
use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

mod app;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 加载 .env 文件中的环境变量
    dotenv().ok();

    // 连接 PostgreSQL 数据库并创建连接池
    let pool = app::db::create_pool().await?;
    println!("数据库已经连接！");

    // Arc 包装，多线程共享
    let shared_pool = Arc::new(pool);

    let mqtt_pool = shared_pool.clone();
    tokio::spawn(async move {
        app::mqtt::run_mqtt(mqtt_pool).await;
    });

    // 设置 CORS 中间件，允许所有来源的 GET 请求（允许来自任何来源的 Web 客户端例如网页前端向此 API 发送请求）
    let cors = CorsLayer::new()
        .allow_methods(Method::GET)
        .allow_origin(Any);

    // 创建路由
    let app = Router::new()
        .route(
            "/data/latest",
            axum::routing::get(app::handlers::get_latest),
        ) // 定义 API 端点（路径）和对应的处理函数
        .route("/data/all", axum::routing::get(app::handlers::get_all))
        .with_state(shared_pool) // 使得 API 处理汉书可以通过 State 提取器访问数据库连接池
        .layer(cors); // 中间件

    println!("REST API running in http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
