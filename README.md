# Rust EC Tutorial - Integration Testing Guide

## 概要
このプロジェクトは、Rust、Kafka、MySQLを使用したマイクロサービスアーキテクチャのECシステムです。

### サービス構成
- **Order Service** (Port: 8080) - 注文管理 (HTTP + Kafka Consumer)
- **Payment Service** (Port: 8081) - 決済処理
- **Inventory Service** (Port: 8082) - 在庫管理 (HTTP + Kafka Consumer)

## 前提条件
- Docker & Docker Compose
- Rust (latest stable)
- cargo

## 結合テストの実行方法

### 1. インフラの起動

```bash
# Docker Composeでインフラを起動
./start-integration-test.sh
```

このスクリプトは以下を実行します:
- MySQL、Kafka、Zookeeperの起動
- Kafkaトピックの作成確認
- サービスのビルド

### 2. サービスの起動

#### オプションA: 自動起動（推奨）

```bash
# 全サービスをバックグラウンドで起動
./run-all-services.sh
```

ログは `logs/` ディレクトリに出力されます:
- `logs/order.log`
- `logs/payment.log`
- `logs/inventory.log`

#### オプションB: 手動起動（デバッグ用）

各サービスを別々のターミナルで起動:

**Terminal 1 - Order Service:**
```bash
export DATABASE_URL="mysql://ecuser:ecpassword@localhost:3306/ec_db"
export KAFKA_BROKERS="localhost:9092"
cargo run --bin order
```

**Terminal 2 - Payment Service:**
```bash
export DATABASE_URL="mysql://ecuser:ecpassword@localhost:3306/ec_db"
export KAFKA_BROKERS="localhost:9092"
cargo run --bin payment
```

**Terminal 3 - Inventory Service:**
```bash
export DATABASE_URL="mysql://ecuser:ecpassword@localhost:3306/ec_db"
export KAFKA_BROKERS="localhost:9092"
cargo run --bin inventory
```

### 3. サービスの停止

```bash
# 全サービスを停止
./stop-all-services.sh

# Dockerサービスも停止する場合
cd etc/docker
docker-compose down
```

## テスト例

### 1. 在庫の追加

まず、在庫を追加します:

```bash
curl -X PUT http://localhost:8082/inventory/p1 \
  -H "Content-Type: application/json" \
  -d '{
    "quantity": 100
  }'
```

### 2. 在庫の確認

```bash
curl http://localhost:8082/inventory/p1
```

レスポンス例:
```json
{
  "product_id": "p1",
  "available_quantity": 100,
  "reserved_quantity": 0
}
```

### 3. 注文の作成

```bash
curl -X POST http://localhost:8080/orders \
  -H "Content-Type: application/json" \
  -d '{
    "customer_id": "c1",
    "product_id": "p1",
    "quantity": 2
  }'
```

レスポンス例:
```json
{
  "id": "09e2aab5-4c26-4e0f-901e-4d0b72d7ec25",
  "customer_id": "c1",
  "status": "AwaitingInventory",
  "products": [...]
}
```

### 4. 注文の取得（ステータス確認）

```bash
curl http://localhost:8080/orders/09e2aab5-4c26-4e0f-901e-4d0b72d7ec25
```

在庫が確保されると、ステータスが `AwaitingPayment` に変わります。

### 5. 決済の処理

```bash
curl -X POST http://localhost:8081/payments \
  -H "Content-Type: application/json" \
  -d '{
    "order_id": "09e2aab5-4c26-4e0f-901e-4d0b72d7ec25",
    "amount": 2000,
    "payment_method": "credit_card"
  }'
```

### 6. 在庫の再確認

決済完了後、在庫が確定されます:

```bash
curl http://localhost:8082/inventory/p1
```

レスポンス例:
```json
{
  "product_id": "p1",
  "available_quantity": 98,
  "reserved_quantity": 0
}
```

## トラブルシューティング

### Kafkaトピックが見つからない場合

```bash
# Kafkaコンテナに入る
cd etc/docker
docker-compose exec kafka bash

# トピック一覧を確認
kafka-topics --list --bootstrap-server localhost:9092

# トピックを手動作成
kafka-topics --create --if-not-exists --bootstrap-server localhost:9092 --topic order-events --partitions 1 --replication-factor 1
kafka-topics --create --if-not-exists --bootstrap-server localhost:9092 --topic payment-events --partitions 1 --replication-factor 1
kafka-topics --create --if-not-exists --bootstrap-server localhost:9092 --topic inventory-events --partitions 1 --replication-factor 1
```

### ポート競合エラー

各サービスが使用するポート:
- Order: 8080
- Payment: 8081
- Inventory: 8082
- MySQL: 3306
- Kafka: 9092

既に使用されている場合は、該当プロセスを停止してください。

### データベース接続エラー

```bash
# MySQLの状態確認
cd etc/docker
docker-compose ps

# MySQLログの確認
docker-compose logs mysql
```

## アーキテクチャ

```
┌─────────────┐      ┌──────────────┐      ┌───────────────┐
│   Order     │─────▶│    Kafka     │─────▶│   Inventory   │
│  Service    │      │              │      │    Service    │
│  (8080)     │      │ order-events │      │  (Consumer)   │
└─────────────┘      └──────────────┘      └───────────────┘
       │                     ▲                      │
       │                     │                      │
       ▼                     │                      ▼
┌─────────────┐      ┌──────────────┐      ┌───────────────┐
│   Payment   │─────▶│    Kafka     │      │     MySQL     │
│  Service    │      │              │      │   Database    │
│  (8081)     │      │payment-events│      │    (3306)     │
└─────────────┘      └──────────────┘      └───────────────┘
```

## 環境変数

| 変数名 | デフォルト値 | 説明 |
|--------|-------------|------|
| DATABASE_URL | mysql://ecuser:ecpassword@localhost:3306/ec_db | MySQL接続文字列 |
| KAFKA_BROKERS | localhost:9092 | Kafkaブローカーアドレス |

## 開発

### ビルド

```bash
cargo build
```

### テスト

```bash
cargo test
```

### フォーマット

```bash
cargo fmt
```

### Lint

```bash
cargo clippy
```
