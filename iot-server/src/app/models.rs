//！ 传感器数据模型
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct SensorData {
    pub device: String,
    pub temp: f64,
    #[serde(default)] // 允许 JSON 数据中缺少该字段，默认值为0.0
    pub humidity: f64,
}

#[derive(Debug, Serialize)]
pub struct SensorRecord {
    pub id: i32,
    pub device_id: String,
    pub temperature: f64,
    pub humidity: f64,
    pub created_at: Option<NaiveDateTime>,
}
