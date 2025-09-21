//！ API 端点处理程序
use axum::{Json, extract::State};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::app::models::SensorRecord;

// 获取最新数据
pub async fn get_latest(State(pool): State<Arc<Pool<Postgres>>>) -> Json<Option<SensorRecord>> {
    let row = sqlx::query!(
        "SELECT id, device_id, temperature, humidity, created_at FROM sensor_data \
        ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(&*pool) // 可能返回一条数据
    .await
    .unwrap();

    // map 函数将查询结果转换为 SensorRecord 结构体
    let record = row.map(|r| SensorRecord {
        id: r.id,
        device_id: r.device_id,
        temperature: r.temperature,
        humidity: r.humidity.unwrap_or(0.0),
        created_at: r.created_at,
    });

    Json(record)
}

// 获取所有数据
pub async fn get_all(State(pool): State<Arc<Pool<Postgres>>>) -> Json<Vec<SensorRecord>> {
    let rows = sqlx::query!(
        "SELECT id, device_id, temperature, humidity, created_at FROM sensor_data ORDER BY created_at DESC"
    )
    .fetch_all(&*pool)
    .await
    .unwrap();

    let records = rows
        .into_iter()
        .map(|r| SensorRecord {
            id: r.id,
            device_id: r.device_id,
            temperature: r.temperature,
            humidity: r.humidity.unwrap_or(0.0),
            created_at: r.created_at,
        })
        .collect();

    Json(records)
}
