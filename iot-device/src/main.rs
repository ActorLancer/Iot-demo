use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Serialize;
use rand::Rng;
use tokio::time::{sleep, Duration};

#[derive(Serialize)]
struct SensorData {
    device: String,
    temp: f64,
    humidity: f64,
}

#[tokio::main]
async fn main() {
    // 
    let mut mqttoptions = MqttOptions::new("iot-device-1", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    //
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(notification) => {
                    println!("MQTT Event: {:?}", notification);
                },
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
        let mut rng = rand::rng();
        let data = SensorData {
            device: "sensor-1".to_string(),
            temp: rng.random_range(20.0..30.0),
            humidity: rng.random_range(40.0..80.0),
        };

        let payload = serde_json::to_string(&data).unwrap();

        match client.publish("sensors/temperature", QoS::AtMostOnce, false, payload).await {
            Ok(_) => println!("Send data to server sccussfully"),
            Err(e) => eprintln!("Send faild: {e:?}"),
        }

        sleep(Duration::from_secs(5)).await;
    }
}
