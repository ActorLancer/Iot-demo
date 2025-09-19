# Iot-demo

## 架构图
```
┌──────────┐       MQTT        ┌───────────────┐       REST API        ┌──────────────┐
│ IoT设备端 │ ───────────────▶ │ MQTT Broker    │ ───────────────────▶ │ IoT 服务端   │
│ (Rust)   │                   │ (Eclipse Mosq) │                       │ (Rust + DB)  │
└──────────┘                   └───────────────┘                       └──────────────┘
      ▲                                                                      │
      │                                                                      ▼
      │                                                               ┌─────────────┐
      │                                                               │ PostgreSQL  │
      │                                                               └─────────────┘
      │
      ▼
┌──────────────┐
│ 前端监控面板 │
│ (Vue/React)  │
└──────────────┘

```

## 环境准备
1. 安装 Rust 工具链
```
sudo apt update
sudo apt install build-essential pkg-config libssl-dev -y

# 安装 rustup（推荐方式）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

2. 安装 Docker（用于运行 MQTT broker 和 PostgreSQL）
```
sudo apt install docker.io docker-compose -y
sudo systemctl enable docker --now
sudo usermod -aG docker $USER
```

3. 启动 MQTT broker（Eclipse Mosquitto）
```
docker run -it -d --name mosquitto \
  -p 1883:1883 \
  -v $PWD/mosquitto.conf:/mosquitto/config/mosquitto.conf \
  eclipse-mosquitto
```
| 需要手动修改配置文件，允许匿名访问（用于测试环境）


4. 启动数据库（PostgreSQL）
```
docker run --name postgres -e POSTGRES_PASSWORD=123456 -p 5432:5432 -d postgres
```

5. 数据库准备
```
# 进入数据库
docker exec -it postgres psql -U postgres
# 创建数据库和表
CREATE DATABASE iot;

\c iot

CREATE TABLE sensor_data (
    id SERIAL PRIMARY KEY,
    device_id VARCHAR(50) NOT NULL,
    temperature FLOAT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
# 退出
\q
```

## 测试
1. 启动订阅
```
mosquitto_sub -h localhost -t test
```

2. 另一个终端，发布：
```
mosquitto_pub -h localhost -t test -m "hello iot"
```
| 订阅端会收到消息

3. 数据库验证
```
docker exec -it postgres psql -U postgres -d iot
SELECT * FROM sensor_data;
```
