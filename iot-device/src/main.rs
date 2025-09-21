//! 使用 rumqttc 库实现一个简单的 IoT 设备，周期性地生成传感器数据并通过 MQTT 协议发送到代理服务器。
//! 设备模拟温度和湿度传感器的数据，并将数据以 JSON 格式发布到指定的主题。

// AsyncClient 负责与 MQTT 代理客户端建立和维护连接、发布和订阅消息
// MqttOptions 用于配置连接参数，如客户端 ID、代理地址和端口
// QoS 定义了消息传递的服务质量等级
use rand::Rng;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Serialize;
// sleep 用于暂停当前异步任务一段时间
use tokio::time::{Duration, sleep};

#[derive(Serialize)]
struct SensorData {
    device: String,
    temp: f64,
    humidity: f64,
}

#[tokio::main]
async fn main() {
    // 1. 连接到 MQTT 代理服务器，这个服务器被指定为 localhost:1884
    // 传入参数包括客户端 ID、代理地址和端口
    // 设置保持连接的时间间隔为 5s
    let mut mqttoptions = MqttOptions::new("iot-device-1", "localhost", 1884);
    // 设置 心跳（keep alive）间隔。这告诉 MQTT 代理，如果设备在 5s 内没有发送任何或接收任何消息，它会发送一个心跳包以确保连接仍然活跃。
    // 如果代理在这个时间段内没有收到任何消息，它可能会认为连接已经断开。
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    // 创建一个异步 MQTT 客户端和事件循环
    // client：用来发布和订阅消息，以及管理与 MQTT 代理的连接
    // eventloop：处理来自代理的事件，如连接状态变化、消息接收等
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    //
    tokio::spawn(async move {
        loop {
            // 轮询事件循环以处理来自 MQTT 代理的事件
            match eventloop.poll().await {
                Ok(notification) => {
                    println!("MQTT Event: {:?}", notification);
                }
                Err(e) => {
                    eprintln!("MQTT ERROR: {:?}", e);
                    break;
                }
            }
        }
    });

    println!("IoT Device is running ...");

    // 2. Send msg in fixed time
    loop {
        // 创建一个随机数生成器实例
        let mut rng = rand::rng();
        let data = SensorData {
            device: "sensor-1".to_string(),
            temp: rng.random_range(20.0..30.0),
            humidity: rng.random_range(40.0..80.0),
        };

        // 将传感器数据序列化为 JSON 字符串
        let payload = serde_json::to_string(&data).unwrap();

        // publish 方法用于将消息发布到指定的主题
        // 传递的参数：Topic（主题）、QoS（服务质量等级）、retain（是否保留消息）和 payload（消息内容）
        match client
            .publish("sensors/temperature", QoS::AtMostOnce, false, payload)
            .await
        {
            Ok(_) => println!("Send data to server sccussfully"),
            Err(e) => eprintln!("Send faild: {e:?}"),
        }

        sleep(Duration::from_secs(5)).await;
    }
}
