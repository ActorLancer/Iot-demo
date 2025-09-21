//! 数据库连接逻辑
use sqlx::{Pool, Postgres};
use std::env;

pub async fn create_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file.");
    let pool = Pool::<Postgres>::connect(&database_url).await?;
    Ok(pool)
}
