use dotenv::dotenv;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::time::Duration;
use std::sync::Arc;
use axum::{
    extract::State,
    Json,
    Router,
    routing::get,
    http::Method,
};
use chrono::NaiveDateTime;
use tower_http::cors::{Any, CorsLayer};

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct SensorData {
    device: String,
    temp: f64,
    #[serde(default)]   // 允许 JSON 数据中缺少该字段，默认值为0.0
    humidity: f64,
}

#[derive(Debug, Serialize)]
struct SensorRecord {
    id: i32,
    device_id: String,
    temperature: f64,
    humidity: f64,
    created_at: Option<NaiveDateTime>,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file.");

    let pool = Pool::<Postgres>::connect(&database_url).await?;
    println!("数据库已经连接！");

    // Arc 包装，多线程共享
    let shared_pool = Arc::new(pool);

    let mqtt_pool = shared_pool.clone();
    tokio::spawn(async move {
        run_mqtt(mqtt_pool).await;
    });

    let cors = CorsLayer::new()
        .allow_methods(Method::GET)
        .allow_origin(Any);

    let app = Router::new()
        .route("/data/latest", get(get_latest))
        .route("/data/all", get(get_all))
        .with_state(shared_pool)
        .layer(cors);

    println!("REST API running in http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// MQTT 消息处理任务
async fn run_mqtt(pool: Arc<Pool<Postgres>>) {
    let mut mqttoptions = MqttOptions::new("iot-server", "localhost", 1883);
    {
        mqttoptions.set_keep_alive(Duration::from_secs(10));
    }

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe("sensors/temperature", QoS::AtMostOnce)
        .await
        .unwrap();

    println!("IoT Server 已启动，等待传感器数据...（正在监听 MQTT 消息）");

    loop {
        match eventloop.poll().await {
            Ok(notification) => match notification {
                Event::Incoming(Packet::Publish(p)) => {
                    let payload_str = String::from_utf8_lossy(&p.payload);
                    println!("收到主题 [{}] 的消息：{}", p.topic, payload_str);

                    // 解析 JSON 数据
                    match serde_json::from_str::<SensorData>(&payload_str) {
                        Ok(data) => {
                            println!("解析结果：{:?}", data);

                            // 插入数据库
                            if let Err(e) = sqlx::query!(
                                "INSERT INTO sensor_data (device_id, temperature, humidity) VALUES ($1, $2, $3)",
                                data.device,
                                data.temp,
                                data.humidity,
                            )
                            .execute(&*pool)
                            .await
                            {
                                eprintln!("数据库插入失败：{:?}", e);
                            } else {
                                println!("数据已经保存！");
                            }
                        },
                        Err(e) => eprintln!("JSON 解析失败：{e:?}"),
                    }
                }
                _ => {}
            },
            Err(e) => {
                eprintln!("Connection Error: {e:?}");
                break;
            }
        }
    }
}

// 获取最新数据
async fn get_latest(State(pool): State<Arc<Pool<Postgres>>>) -> Json<Option<SensorRecord>> {
    let row = sqlx::query!(
        "SELECT id, device_id, temperature, humidity, created_at FROM sensor_data ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(&*pool)
    .await
    .unwrap();

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
async fn get_all(State(pool): State<Arc<Pool<Postgres>>>) -> Json<Vec<SensorRecord>> {
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
