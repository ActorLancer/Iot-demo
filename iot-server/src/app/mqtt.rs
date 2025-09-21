//! MQTT 客户端和消息处理
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use std::time::Duration;

use crate::app::models::SensorData;

// MQTT 消息处理任务
pub async fn run_mqtt(pool: Arc<Pool<Postgres>>) {
    let mut mqttoptions = MqttOptions::new("iot-server", "localhost", 1884);
    mqttoptions.set_keep_alive(Duration::from_secs(10));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe("sensors/temperature", QoS::AtMostOnce)
        .await
        .unwrap();

    println!("IoT Server 已启动，等待传感器数据...（正在监听 MQTT 消息）");

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(p))) => {
                let payload_str = String::from_utf8_lossy(&p.payload);
                println!("收到主题 [{}] 的消息：{}", p.topic, payload_str);

                match serde_json::from_str::<SensorData>(&payload_str) {
                    Ok(data) => {
                        println!("解析结果：{:?}", data);

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
                    }
                    Err(e) => eprintln!("JSON 解析失败：{e:?}"),
                }
            }
            Err(e) => {
                eprintln!("Connection Error: {e:?}");
                break;
            }
            _ => {}
        }
    }
}
