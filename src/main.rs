use dotenv::dotenv;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use tokio::time;

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct SensorData {
    device: String,
    temp: f32,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file.");

    let pool = Pool::<Postgres>::connect(&database_url).await?;
    println!("数据库已经连接！");

    // 1. 配置 MQTT 客户端
    let mut mqttotions = MqttOptions::new("iot-server", "localhost", 1883);
    mqttotions.set_keep_alive(Duration::from_secs(10));

    let (client, mut eventloop) = AsyncClient::new(mqttotions, 10);

    // 2. 订阅一个主题，比如传感器数据
    client
        .subscribe("sensors/temperature", QoS::AtMostOnce)
        .await
        .unwrap();

    println!("IOT Server 已启动，正在监听 MQTT 消息...");

    // 3. 异步循环读取消息
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(notification) => match notification {
                    Event::Incoming(Packet::Publish(p)) => {
                        let payload_str = String::from_utf8_lossy(&p.payload);
                        println!("收到主题 [{}] 的消息: {}", p.topic, payload_str,);

                        // 解析 JSON 数据
                        match serde_json::from_str::<SensorData>(&payload_str) {
                            Ok(data) => {
                                println!("解析结果：{:?}", data);

                                // 插入数据库
                                if let Err(e) = sqlx::query!(
                                    "INSERT INTO sensor_data (device_id, temperature) VALUES ($1, $2)",
                                    data.device,
                                    data.temp as f64
                                )
                                .execute(&pool_clone)
                                .await
                                {
                                    eprintln!("数据库插入数据失败：{:?}", e);
                                } else {
                                    println!("数据已成功保存");
                                }
                            }
                            Err(e) => {
                                eprintln!("JSON 解析失败：{:?}", e);
                            }
                        }
                    }
                    _ => {}
                },
                Err(e) => {
                    eprintln!("连接错误: {:?}", e);
                    break;
                }
            }
        }
    });

    // 4. 让 main 不退出
    loop {
        time::sleep(Duration::from_secs(60)).await;
    }
}
